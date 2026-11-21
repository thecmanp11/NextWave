import os
import json
import subprocess
from datetime import datetime, timedelta, timezone
import numpy as np
import pandas as pd
import xarray as xr
import requests

import copernicusmarine
from spots import SPOTS

# TODO: from google.cloud import firestore

# space
TZ = "Europe/Athens"
# GREECE
BBOX = dict(minimum_longitude=18.0, maximum_longitude=30.5,minimum_latitude=33.5,  maximum_latitude=42.0)

# time
now = datetime.now(timezone.utc).replace(minute=0, second=0, microsecond=0, tzinfo=None)
START, END = now - timedelta(hours=6), now + timedelta(days=8)

# Extract a point (snaps to nearest sea cell, since coastal cells can be NaN)
def nearest_ocean_cell(da2d, lat, lon):
    lats, lons = np.meshgrid(da2d.latitude.values, da2d.longitude.values, indexing="ij")
    dist = np.hypot(lats - lat, (lons - lon) * np.cos(np.radians(lat)))
    dist = np.where(np.isnan(da2d.values), np.inf, dist)
    return np.unravel_index(dist.argmin(), dist.shape)

# TODO: redundant .load() per spot - move outside, run once and pass it in
def wave_point(ds, lat, lon, variables):
    first = ds["VHM0"].isel(time=0).load()
    i, j = nearest_ocean_cell(first, lat, lon)
    df = ds[variables].isel(latitude=i, longitude=j).to_dataframe()
    df = df.drop(columns=[c for c in ("latitude", "longitude") if c in df.columns])
    df.index = pd.to_datetime(df.index).tz_localize("UTC").tz_convert(TZ)
    df.index.name = "time"
    return df

# Wind from Open-Meteo
def get_wind(lat, lon, days=8, models=None):
    params = dict(
        latitude=lat, longitude=lon,
        hourly="wind_speed_10m,wind_direction_10m,wind_gusts_10m",
        wind_speed_unit="kn", timezone="UTC", forecast_days=days,
    )
    if models:  # e.g. "icon_d2", "ecmwf_ifs025", "gfs_seamless"
        params["models"] = models
    r = requests.get("https://api.open-meteo.com/v1/forecast", params=params, timeout=30)
    r.raise_for_status()
    df = pd.DataFrame(r.json()["hourly"])
    df["time"] = pd.to_datetime(df["time"], utc=True)
    return (df.set_index("time").tz_convert(TZ)
              .rename(columns={"wind_speed_10m": "wind_kn",
                               "wind_direction_10m": "wind_dir",
                               "wind_gusts_10m": "gust_kn"}))

# Merge and derive per-spot fields
def angdiff(a, b):
    """Signed smallest angle a-b in degrees, [-180, 180)."""
    return (a - b + 180) % 360 - 180

def build_frame(waves, wind, spot):
    df = waves.join(wind, how="inner")
    df["swell_off"] = angdiff(df["VMDR"], spot["facing"]).abs()
    df["in_window"] = df["swell_off"] <= spot["window"]

    # TODO: not sure if this is passed into Rust, I think it's duplicated there...
    w_off = angdiff(df["wind_dir"], spot["facing"]).abs()
    df["wind_rel"] = np.where(w_off >= 120, "offshore",
                      np.where(w_off <= 60, "onshore", "cross"))

    tm = df["VTM10"] if "VTM10" in df else df["VTPK"]
    if spot.get("use_wind_waves"):
        hs, per = df["VHM0"], tm
    else:
        hs = df["VHM0_SW1"].fillna(df["VHM0"])
        per = df["VTM01_SW1"].fillna(tm)
    # df["power_kw_m"] = 0.49 * df["VHM0"] ** 2 * tm     # deep-water power, relative use only
    df["power_kw_m"] = 0.49 * hs ** 2 * per
    df["swell_share"] = df.get("VHM0_SW1", np.nan) / df["VHM0"]  # swell vs wind-wave mix
    return df

def in_dataset(ds, lat, lon, margin=0.1):
    """True if the point lies inside the downloaded box (with a small margin)."""
    return (ds.latitude.min() + margin <= lat <= ds.latitude.max() - margin and
            ds.longitude.min() + margin <= lon <= ds.longitude.max() - margin)


def main():
    # Copernicus AUTH
    # TODO: set up login as GCP secret
    copernicusmarine.login()  # prompts once, stores creds locally

    # WAVE DATA
    cat = copernicusmarine.describe(product_id="MEDSEA_ANALYSISFORECAST_WAV_006_017")
    for p in cat.products:
        for d in p.datasets:
            print(d.dataset_id)

    # cmems_mod_med_wav_anfc_4.2km_PT1H-i
    # cmems_mod_med_wav_anfc_4.2km_static
    WAVE_DATASET_ID = "cmems_mod_med_wav_anfc_4.2km_PT1H-i"
    # WAVE_VARS = ["VHM0", "VTPK", "VTM10", "VMDR",
    #          "VHM0_WW", "VTM01_WW", "VMDR_WW",
    #          "VHM0_SW1", "VTM01_SW1", "VMDR_SW1"]
    WAVE_VARS = [
        # Total sea
        "VHM0", "VTPK", "VTM10", "VMDR",
        # Wind sea
        "VHM0_WW","VTM01_WW","VMDR_WW",
        # Primary swell
        "VHM0_SW1", "VTM01_SW1", "VMDR_SW1",
        # Secondary swell
        "VHM0_SW2", "VTM01_SW2","VMDR_SW2",
    ]

    STATIC_DATASET_ID = "cmems_mod_med_wav_anfc_4.2km_static"


    # TODO: SKIP DOWNLOAD while testing!!!!
    # Download once to a local NetCDF. Tight bbox around your spots keeps it small.
    copernicusmarine.subset(
        dataset_id=STATIC_DATASET_ID,
        output_filename="static.nc",
        output_directory="data",
        overwrite=True,
        **BBOX,
    )
    copernicusmarine.subset(
        dataset_id=WAVE_DATASET_ID,
        variables=WAVE_VARS,
        start_datetime=START.strftime("%Y-%m-%dT%H:%M:%S"),
        end_datetime=END.strftime("%Y-%m-%dT%H:%M:%S"),
        output_filename="waves.nc",
        output_directory="data",
        overwrite=True,
        **BBOX,
    )

    # local in-mem file downloaded data
    # ds = xr.open_dataset("data/waves.nc").load()
    # show all variables (will just be the ones filtered from my list)
    # ds.data_vars
    with xr.open_dataset("data/static.nc") as s_raw:
        stc = s_raw.load()
    with xr.open_dataset("data/waves.nc") as w_raw:
        ds = w_raw.load()

    frames = {}
    for name, s in SPOTS.items():
        if not in_dataset(ds, s["lat"], s["lon"]):
            print(f"skip {name}: outside the downloaded box")
            continue
        w = wave_point(ds, s["lat"], s["lon"], WAVE_VARS)
        wi = get_wind(s["lat"], s["lon"])
        frames[name] = build_frame(w, wi, s)

    # DUMP to Rust Surf Engine
    # spots (tuples become JSON arrays automatically)
    # json.dump(SPOTS, open("surf_engine/spots.json", "w"))

    # forecasts: drop NaN rows, one list of hourly records per spot
    need = ["VHM0", "VTPK", "VTM10", "VMDR", "wind_kn", "wind_dir", "power_kw_m"]
    forecast = {
        name: json.loads(df.dropna(subset=need).reset_index().to_json(orient="records", date_format="iso"))
        for name, df in frames.items()
    }
    # json.dump(forecast, open("surf_engine/forecast.json", "w"))
    os.makedirs("surf_engine", exist_ok=True)
    with open("surf_engine/spots.json", "w", encoding="utf-8") as f:
        json.dump(SPOTS, f)
    with open("surf_engine/forecast.json", "w", encoding="utf-8") as f:
        json.dump(forecast, f)

    # Run Rust engine via subprocess
    print("Running Rust surf engine...")
    out = subprocess.run(
        ["cargo", "run", "--release", "--quiet", "--", "forecast.json"],
        cwd="surf_engine",
        capture_output=True,
        text=True,
        check=True
    )

    print("STDOUT:", repr(out.stdout))
    print("STDERR:", repr(out.stderr))

    # Assuming Rust prints valid JSON to stdout
    results = json.loads(out.stdout)
    print("Rust engine finished successfully.")
    print(results)

    # TODO: temp write locally before firebase is implemented
    # json.dump(results, open("local_results.json", "w"))
    with open("local_results.json", "w", encoding="utf-8") as f:
        json.dump(json.loads(out.stdout), f, indent=2)

    # TODO: write to firestore
    # Write to Firestore
    # db = firestore.Client()
    # batch = db.batch()
    
    # # Store daily snapshot/forecast documents
    # today_str = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    # doc_ref = db.collection("forecasts").document(today_str)
    # batch.set(doc_ref, {"timestamp": datetime.now(timezone.utc).isoformat(), "data": results})
    # batch.commit()
    # print("Successfully wrote forecast data to Firestore!")


if __name__ == "__main__":
    main()