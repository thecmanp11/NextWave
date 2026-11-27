import json
import os
import subprocess
from datetime import datetime, timedelta, timezone

import numpy as np
import pandas as pd
import math
import requests
import xarray as xr

import copernicusmarine
from spots import SPOTS

BBOX = dict(
    minimum_longitude=18.0,
    maximum_longitude=30.5,
    minimum_latitude=33.5,
    maximum_latitude=42.0,
)

WAVE_DATASET_ID = "cmems_mod_med_wav_anfc_4.2km_PT1H-i"
STATIC_DATASET_ID = "cmems_mod_med_wav_anfc_4.2km_static"

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

now = datetime.now(timezone.utc).replace(minute=0, second=0, microsecond=0)
START, END = now - timedelta(hours=6), now + timedelta(days=7)
FORECAST_DAYS = max(1, math.ceil((END - START).total_seconds() / 86400.0))


def ensure_auth() -> None:
    user = os.getenv("COPERNICUSMARINE_SERVICE_USERNAME")
    pwd = os.getenv("COPERNICUSMARINE_SERVICE_PASSWORD")
    if user and pwd:
        copernicusmarine.login(username=user, password=pwd, check_credentials_valid=False)
    else:
        copernicusmarine.login()


def nearest_ocean_cell(mask_or_wave_2d: xr.DataArray, lat: float, lon: float) -> tuple[int, int]:
    lats, lons = np.meshgrid(
        mask_or_wave_2d.latitude.values,
        mask_or_wave_2d.longitude.values,
        indexing="ij",
    )
    dist = np.hypot(lats - lat, (lons - lon) * np.cos(np.radians(lat)))
    invalid = np.isnan(mask_or_wave_2d.values) | (mask_or_wave_2d.values <= 0)
    dist = np.where(invalid, np.inf, dist)
    i, j = np.unravel_index(dist.argmin(), dist.shape)
    return int(i), int(j)


def extract_static_depth(static_ds: xr.Dataset, lat: float, lon: float) -> tuple[int, int, float | None]:
    depth_var = next((v for v in ("deptho", "depth", "bathymetry", "h") if v in static_ds), None)
    ref = static_ds[depth_var] if depth_var else static_ds[next(iter(static_ds.data_vars))]
    if "time" in ref.dims:
        ref = ref.isel(time=0)
    i, j = nearest_ocean_cell(ref, lat, lon)
    val = float(ref.isel(latitude=i, longitude=j).values) if depth_var else None
    if val is not None and (np.isnan(val) or val <= 0):
        val = None
    return i, j, val


def get_waves_at_cell(ds: xr.Dataset, i: int, j: int) -> pd.DataFrame:
    df = ds[WAVE_VARS].isel(latitude=i, longitude=j).to_dataframe()
    # df = df.drop(columns=[c for c in ("latitude", "longitude") if c in df.columns])
    # idx = pd.to_datetime(df.index)
    # df.index.name = "time"
    # return df
    df = df.drop(columns=["latitude", "longitude"],errors="ignore")
    # specify utc awareness
    df.index = pd.to_datetime(df.index, utc=True)
    df.index.name = "time"
    return df.sort_index()


def get_wind(lat: float, lon: float, days: int = FORECAST_DAYS) -> pd.DataFrame:
    params = dict(
        latitude=lat,
        longitude=lon,
        hourly="wind_speed_10m,wind_direction_10m,wind_gusts_10m",
        wind_speed_unit="kn",
        timezone="UTC",
        forecast_days=days,
    )
    r = requests.get("https://api.open-meteo.com/v1/forecast", params=params, timeout=30)
    r.raise_for_status()
    df = pd.DataFrame(r.json()["hourly"])
    df["time"] = pd.to_datetime(df["time"], utc=True)
    return (
        df.set_index("time")
        .rename(
            columns={
                "wind_speed_10m": "wind_kn",
                "wind_direction_10m": "wind_dir",
                "wind_gusts_10m": "wind_gust_kn",
            }
        )
    )

def opt_float(val) -> float | None:
    return None if val is None or pd.isna(val) else float(val)

def opt_or_fallback(primary, fallback) -> float | None:
    p = opt_float(primary)
    return p if p is not None else opt_float(fallback)

def to_hour_records(df: pd.DataFrame, depth_m: float | None) -> list[dict]:
    records = []
    for ts, row in df.iterrows():
        if pd.isna(row.get("VHM0")) or pd.isna(row.get("wind_kn")) or pd.isna(row.get("wind_dir")):
            continue
        records.append({
            "time": ts.isoformat(),
            "depth_m": depth_m,
            "total": {
                "hs": opt_float(row.get("VHM0")),
                "period": opt_or_fallback(row.get("VTM10"), row.get("VTPK")),
                "direction": opt_float(row.get("VMDR")),
            },
            "wind_sea": {
                "hs": opt_float(row.get("VHM0_WW")),
                "period": opt_float(row.get("VTM01_WW")),
                "direction": opt_float(row.get("VMDR_WW")),
            },
            "primary_swell": {
                "hs": opt_float(row.get("VHM0_SW1")),
                # "period": opt_float(row.get("VTM01_SW1")),
                "period": opt_or_fallback(row.get("VTM01_SW1"), row.get("VTM10")),
                "direction": opt_float(row.get("VMDR_SW1")),
            },
            "secondary_swell": {
                "hs": opt_float(row.get("VHM0_SW2")),
                # "period": opt_float(row.get("VTM01_SW2")),
                "period": opt_or_fallback(row.get("VTM01_SW2"), row.get("VTM10")),
                "direction": opt_float(row.get("VMDR_SW2")),
            },
            "wind_kn": float(row["wind_kn"]),
            "wind_gust_kn": opt_float(row.get("wind_gust_kn")),
            "wind_dir": float(row["wind_dir"]),
        })
    return records


def main() -> None:
    ensure_auth()

    # copernicusmarine.subset(
    #     dataset_id=STATIC_DATASET_ID,
    #     output_filename="static.nc",
    #     output_directory="data",
    #     overwrite=True,
    #     **BBOX,
    # )
    # copernicusmarine.subset(
    #     dataset_id=WAVE_DATASET_ID,
    #     variables=WAVE_VARS,
    #     start_datetime=START.strftime("%Y-%m-%dT%H:%M:%SZ"),
    #     end_datetime=END.strftime("%Y-%m-%dT%H:%M:%SZ"),
    #     output_filename="waves.nc",
    #     output_directory="data",
    #     overwrite=True,
    #     **BBOX,
    # )

    with xr.open_dataset("data/static.nc") as s_raw:
        static_ds = s_raw.load()
    with xr.open_dataset("data/waves.nc") as w_raw:
        wave_ds = w_raw.load()

    forecast: dict[str, list[dict]] = {}
    for name, spot in SPOTS.items():
        i, j, depth_m = extract_static_depth(static_ds, spot["lat"], spot["lon"])
        w_df = get_waves_at_cell(wave_ds, i, j)
        wi_df = get_wind(spot["lat"], spot["lon"]).sort_index()
        merged = w_df.join(wi_df, how="inner")
        forecast[name] = to_hour_records(merged, depth_m)

    os.makedirs("surf_engine", exist_ok=True)
    with open("surf_engine/spots.json", "w", encoding="utf-8") as f:
        json.dump(SPOTS, f)
    with open("surf_engine/forecast.json", "w", encoding="utf-8") as f:
        json.dump(forecast, f)

    out = subprocess.run(
        ["cargo", "run", "--release", "--quiet", "--", "forecast.json"],
        cwd="surf_engine",
        capture_output=True,
        text=True,
        check=True,
    )

    # with open("surf_engine/local_results.json", "w", encoding="utf-8") as f:
    #     json.dump(json.loads(out.stdout), f, indent=2)

    results = json.loads(out.stdout)  # {spot_name: [event, ...]}

    # Flat list of surf events; each event already carries its own "spot" field
    events = [event for spot_events in results.values() for event in spot_events]

    # chronological order across all spots
    # events.sort(key=lambda e: e["start"])

    with open("surf_engine/local_results.json", "w", encoding="utf-8") as f:
        json.dump(events, f, indent=2)


if __name__ == "__main__":
    main()