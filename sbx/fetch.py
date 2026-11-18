# fetch.py: raw data in, payload.json out. No calculations.
import json
from datetime import datetime, timedelta, timezone

import copernicusmarine
import numpy as np
import pandas as pd
import requests
import xarray as xr

from spots import SPOTS

BBOX = dict(minimum_longitude=18.0, maximum_longitude=30.5,minimum_latitude=33.5, maximum_latitude=42.0)
WAVE_DATASET_ID = "cmems_mod_med_wav_anfc_4.2km_PT1H-i"
WAVE_VARS = ["VHM0", "VTPK", "VTM10", "VMDR",
             "VHM0_WW", "VTM01_WW", "VMDR_WW",
             "VHM0_SW1", "VTM01_SW1", "VMDR_SW1",
             "VHM0_SW2", "VTM01_SW2", "VMDR_SW2"]
WIND_MODELS = ["ecmwf_ifs025", "icon_eu", "gfs_seamless"]   # TODO: check ids in the Open-Meteo docs

now = datetime.now(timezone.utc).replace(minute=0, second=0, microsecond=0, tzinfo=None)
START, END = now - timedelta(hours=6), now + timedelta(days=7)


def records(df):
    """Hourly rows with UTC ISO times; NaN becomes null."""
    df = df.copy()
    df.index = pd.to_datetime(df.index, utc=True)
    df = df.reset_index(names="time")
    df["time"] = df["time"].dt.strftime("%Y-%m-%dT%H:%M:%SZ")
    return json.loads(df.to_json(orient="records"))


def get_waves():
    remote = copernicusmarine.open_dataset(dataset_id=WAVE_DATASET_ID)      # metadata only
    variables = [v for v in WAVE_VARS if v in remote.data_vars]
    missing_vars = [v for v in WAVE_VARS if v not in variables]
    if missing_vars:
        print("missing wave vars:", [v for v in WAVE_VARS if v not in variables])
    copernicusmarine.subset(
        dataset_id=WAVE_DATASET_ID, variables=variables,
        start_datetime=START.strftime("%Y-%m-%dT%H:%M:%S"),
        end_datetime=END.strftime("%Y-%m-%dT%H:%M:%S"),
        output_filename="waves.nc", output_directory="data", overwrite=True, **BBOX)
    return xr.open_dataset("data/waves.nc").load(), variables


def wave_series(ds, variables, lat, lon):
    """Raw series for the nearest sea cell, plus how far that cell is from the spot."""
    first = ds["VHM0"].isel(time=0)
    lats, lons = np.meshgrid(first.latitude.values, first.longitude.values, indexing="ij")
    d = np.hypot(lats - lat, (lons - lon) * np.cos(np.radians(lat)))
    d = np.where(np.isnan(first.values), np.inf, d)
    i, j = np.unravel_index(d.argmin(), d.shape)
    cell = dict(lat=float(first.latitude[i]), lon=float(first.longitude[j]),
                dist_km=float(111.2 * d[i, j]))
    df = (ds[variables].isel(latitude=i, longitude=j).to_dataframe()
            .drop(columns=["latitude", "longitude"], errors="ignore"))
    return cell, records(df)


def openmeteo(lat, lon, model):
    r = requests.get("https://api.open-meteo.com/v1/forecast", timeout=30, params=dict(
        latitude=lat, longitude=lon, models=model, timezone="UTC", forecast_days=8,
        wind_speed_unit="kn",
        hourly="wind_speed_10m,wind_direction_10m,wind_gusts_10m,temperature_2m,"
               "precipitation,cloud_cover,pressure_msl,weather_code"))
    r.raise_for_status()
    df = pd.DataFrame(r.json()["hourly"]).set_index("time")
    return records(df.rename(columns={"wind_speed_10m": "wind_kn",
                                      "wind_direction_10m": "wind_dir",
                                      "wind_gusts_10m": "gust_kn"}))


def marine(lat, lon):
    """Second wave opinion + sea temperature. Optional: [] if it fails."""
    try:
        r = requests.get("https://marine-api.open-meteo.com/v1/marine", timeout=30, params=dict(
            latitude=lat, longitude=lon, timezone="UTC", forecast_days=8,
            hourly="wave_height,sea_surface_temperature"))
        r.raise_for_status()
        return records(pd.DataFrame(r.json()["hourly"]).set_index("time"))
    except requests.RequestException as e:
        print("marine failed:", e)
        return []


def main():
    ds, variables = get_waves()
    spots = {}
    for sid, cfg in SPOTS.items():
        lat, lon = cfg["lat"], cfg["lon"]
        if not (float(ds.latitude.min()) <= lat <= float(ds.latitude.max())
                and float(ds.longitude.min()) <= lon <= float(ds.longitude.max())):
            print(f"skip {sid}: outside the downloaded box")
            continue
        cell, waves = wave_series(ds, variables, lat, lon)
        wind = {}
        for m in WIND_MODELS:
            try:
                wind[m] = openmeteo(lat, lon, m)
            except requests.RequestException as e:
                print(f"{sid}: {m} failed: {e}")
        spots[sid] = dict(config=cfg, cell=cell, waves=waves, wind=wind, marine=marine(lat, lon))

    payload = dict(generated_at=now.isoformat() + "Z", spots=spots)
    with open("surf_engine2/payload.json", "w") as f:
        json.dump(payload, f)

    # later: run the engine from here
    # import subprocess
    # out = subprocess.run(["cargo", "run", "--release", "--quiet", "--", "payload.json"],
    #                      cwd="surf_engine", capture_output=True, text=True, check=True)
    # result = json.loads(out.stdout)


if __name__ == "__main__":
    main()