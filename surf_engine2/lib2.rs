//! Surf engine: payload.json in, decisions out.
//! Python only fetches raw data. Everything below is calculation.
//!
//! Sections:
//!   1. Config (Spot)
//!   2. Payload types (raw data from Python)
//!   3. Aligned hour (Hour) + build_hours
//!   4. Maths helpers (angles, sun)
//!   5. Tier 1 + 3: evaluate (gates, factors, score, confidence)
//!   6. Tier 2: sessions
//!   7. run(): the single entry point (wrap this with PyO3 later)
//!   8. Tests

use chrono::{DateTime, Datelike, FixedOffset, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 1. CONFIG: one spot, exactly the dict you define in spots.py
// ============================================================================
#[derive(Deserialize, Clone)]
pub struct Spot {
    pub lat: f64,
    pub lon: f64,
    pub facing: f64,
    pub window: f64,
    pub min_hs: f64,
    pub max_hs: f64,
    pub min_period: f64,
    pub ideal_period: (f64, f64),
    pub offshore_dir: (f64, f64),
    pub max_onshore_kn: f64,
    pub max_offshore_kn: f64,
    #[serde(default)]
    pub use_wind_waves: bool,
    pub ideal_swell_dir: Option<(f64, f64)>,
    pub driver_wind_dir: Option<(f64, f64)>,
    pub min_driver_wind_kn: Option<f64>,
    pub season_months: Option<(u32, u32)>,
    pub min_power_kw_m: Option<f64>,
    pub ideal_power_kw_m: Option<f64>,
}

// ============================================================================
// 2. PAYLOAD: raw data as Python sends it. Every raw field is Option so a
//    NaN (null) never crashes the parse; incomplete hours are skipped later.
// ============================================================================
#[derive(Deserialize)]
pub struct Payload {
    pub generated_at: DateTime<FixedOffset>,
    pub spots: HashMap<String, SpotInput>,
}

#[derive(Deserialize)]
pub struct SpotInput {
    pub config: Spot,
    pub cell: Cell,
    pub waves: Vec<WaveRow>,
    pub wind: HashMap<String, Vec<WindRow>>, // model name -> hourly rows
    #[serde(default)]
    pub marine: Vec<MarineRow>,
}

#[derive(Deserialize)]
pub struct Cell {
    pub dist_km: f64, // distance from the spot to the model cell used
}

#[derive(Deserialize)]
pub struct WaveRow {
    pub time: DateTime<FixedOffset>,
    #[serde(rename = "VHM0")]
    pub hs: Option<f64>,
    #[serde(rename = "VTPK")]
    pub tp: Option<f64>,
    #[serde(rename = "VTM10")]
    pub tm10: Option<f64>,
    #[serde(rename = "VMDR")]
    pub dir: Option<f64>,
    #[serde(rename = "VHM0_WW")]
    pub ww_hs: Option<f64>,
    #[serde(rename = "VTM01_WW")]
    pub ww_tm01: Option<f64>,
    #[serde(rename = "VMDR_WW")]
    pub ww_dir: Option<f64>,
    #[serde(rename = "VHM0_SW1")]
    pub sw1_hs: Option<f64>,
    #[serde(rename = "VTM01_SW1")]
    pub sw1_tm01: Option<f64>,
    #[serde(rename = "VMDR_SW1")]
    pub sw1_dir: Option<f64>,
    #[serde(rename = "VHM0_SW2")]
    pub sw2_hs: Option<f64>,
    #[serde(rename = "VTM01_SW2")]
    pub sw2_tm01: Option<f64>,
    #[serde(rename = "VMDR_SW2")]
    pub sw2_dir: Option<f64>,
}

#[derive(Deserialize)]
pub struct WindRow {
    pub time: DateTime<FixedOffset>,
    pub wind_kn: Option<f64>,
    pub wind_dir: Option<f64>,
    pub gust_kn: Option<f64>,
    pub precipitation: Option<f64>,
    pub temperature_2m: Option<f64>,
    pub weather_code: Option<f64>,
}

#[derive(Deserialize)]
pub struct MarineRow {
    pub time: DateTime<FixedOffset>,
    pub wave_height: Option<f64>,
    pub sea_surface_temperature: Option<f64>,
}

// ============================================================================
// 3. ALIGNED HOUR: everything we know about one hour, all sources merged
// ============================================================================
/// One wave partition (wind sea or a swell).
#[derive(Clone, Copy)]
pub struct Part {
    pub hs: f64,
    pub tm01: f64,
    pub dir: f64,
}

#[derive(Clone)]
pub struct Hour {
    pub time: DateTime<FixedOffset>,
    // total sea state (Copernicus)
    pub hs: f64,
    pub tp: f64, // peak period (VTPK): the number Surfline/Windy show
    pub tm10: f64, // energy period: used for power
    pub wave_dir: f64,
    // partitions
    pub ww: Option<Part>,
    pub sw1: Option<Part>,
    pub sw2: Option<Part>,
    // wind: consensus of all models
    pub wind_kn: f64,
    pub wind_dir: f64,
    pub gust_kn: Option<f64>,
    pub wind_spread_kn: f64,  // model disagreement, speed
    pub wind_spread_deg: f64, // model disagreement, direction
    // atmosphere / sea
    pub precip_mm: Option<f64>,
    pub weather_code: Option<f64>,
    pub air_c: Option<f64>,
    pub sea_c: Option<f64>,
    pub hs_alt: Option<f64>, // second wave model, for agreement
    pub cell_km: f64,
}

fn part(hs: Option<f64>, tm01: Option<f64>, dir: Option<f64>) -> Option<Part> {
    Some(Part { hs: hs?, tm01: tm01?, dir: dir? })
}

fn mean(it: impl Iterator<Item = f64>) -> Option<f64> {
    let (s, n) = it.fold((0.0, 0usize), |(s, n), x| (s + x, n + 1));
    if n > 0 { Some(s / n as f64) } else { None }
}

/// Speed-weighted vector mean of the wind models, plus how far they disagree.
/// Returns (speed_kn, dir_deg, speed_spread_kn, dir_spread_deg).
fn wind_consensus(rows: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    let n = rows.len() as f64;
    let (mut u, mut v, mut s) = (0.0, 0.0, 0.0);
    for &(kn, d) in rows {
        let r = d.to_radians();
        u += kn * r.sin();
        v += kn * r.cos();
        s += kn;
    }
    let dir = u.atan2(v).to_degrees().rem_euclid(360.0);
    let kn = s / n;
    let spread_kn = (rows.iter().map(|(k, _)| (k - kn).powi(2)).sum::<f64>() / n).sqrt();
    let spread_deg = rows.iter().map(|(_, d)| angdiff(*d, dir).abs()).fold(0.0, f64::max);
    (kn, dir, spread_kn, spread_deg)
}

/// Merge every source by timestamp. Hours missing core wave or wind data are dropped.
pub fn build_hours(s: &SpotInput) -> Vec<Hour> {
    let wind: Vec<HashMap<i64, &WindRow>> = s
        .wind
        .values()
        .map(|rows| rows.iter().map(|r| (r.time.timestamp(), r)).collect())
        .collect();
    let marine: HashMap<i64, &MarineRow> =
        s.marine.iter().map(|r| (r.time.timestamp(), r)).collect();

    s.waves
        .iter()
        .filter_map(|w| {
            let t = w.time.timestamp();
            let (hs, tp, tm10, wave_dir) = (w.hs?, w.tp?, w.tm10?, w.dir?);
            let rows: Vec<&WindRow> = wind.iter().filter_map(|m| m.get(&t).copied()).collect();
            let pairs: Vec<(f64, f64)> = rows
                .iter()
                .filter_map(|r| Some((r.wind_kn?, r.wind_dir?)))
                .collect();
            if pairs.is_empty() {
                return None;
            }
            let (wind_kn, wind_dir, wind_spread_kn, wind_spread_deg) = wind_consensus(&pairs);
            let m = marine.get(&t);
            Some(Hour {
                time: w.time,
                hs,
                tp,
                tm10,
                wave_dir,
                ww: part(w.ww_hs, w.ww_tm01, w.ww_dir),
                sw1: part(w.sw1_hs, w.sw1_tm01, w.sw1_dir),
                sw2: part(w.sw2_hs, w.sw2_tm01, w.sw2_dir),
                wind_kn,
                wind_dir,
                gust_kn: mean(rows.iter().filter_map(|r| r.gust_kn)),
                wind_spread_kn,
                wind_spread_deg,
                precip_mm: mean(rows.iter().filter_map(|r| r.precipitation)),
                weather_code: rows.iter().filter_map(|r| r.weather_code).reduce(f64::max),
                air_c: mean(rows.iter().filter_map(|r| r.temperature_2m)),
                sea_c: m.and_then(|m| m.sea_surface_temperature),
                hs_alt: m.and_then(|m| m.wave_height),
                cell_km: s.cell.dist_km,
            })
        })
        .collect()
}

fn part_power(p: &Part) -> f64 {
    0.49 * p.hs * p.hs * p.tm01
}

impl Hour {
    /// Deep-water wave power, kW per metre of wave front (relative guide, not surf-zone power).
    /// Wind-sea spots use total energy; swell spots add up the swell partitions.
    pub fn power_for(&self, spot: &Spot) -> f64 {
        let total = 0.49 * self.hs * self.hs * self.tm10;
        if spot.use_wind_waves {
            return total;
        }
        match self.sw1 {
            Some(a) => part_power(&a) + self.sw2.map(|b| part_power(&b)).unwrap_or(0.0),
            None => total,
        }
    }
}

// ============================================================================
// 4. MATHS HELPERS (all directions are "coming from")
// ============================================================================
#[derive(Clone, Copy)]
enum WindRel {
    Offshore,
    Cross,
    Onshore,
}

fn angdiff(a: f64, b: f64) -> f64 {
    (a - b + 540.0).rem_euclid(360.0) - 180.0
}

// true if d lies in the arc from..to, handling wrap through 0
fn in_arc(d: f64, (from, to): (f64, f64)) -> bool {
    let (d, from, to) = (d.rem_euclid(360.0), from.rem_euclid(360.0), to.rem_euclid(360.0));
    if from <= to {
        d >= from && d <= to
    } else {
        d >= from || d <= to
    }
}

// middle of an arc, handling wrap: (235, 285) -> 260, (340, 30) -> 5
fn arc_mid((from, to): (f64, f64)) -> f64 {
    (from + (to - from).rem_euclid(360.0) / 2.0).rem_euclid(360.0)
}

fn wind_rel(spot: &Spot, h: &Hour) -> WindRel {
    if in_arc(h.wind_dir, spot.offshore_dir) {
        WindRel::Offshore
    } else if angdiff(h.wind_dir, spot.facing).abs() <= 60.0 {
        WindRel::Onshore
    } else {
        WindRel::Cross
    }
}

fn ramp(x: f64, lo: f64, hi: f64) -> f64 {
    ((x - lo) / (hi - lo)).clamp(0.0, 1.0)
}

// 1.0 inside [lo, hi], falling linearly to 0 at `soft` distance outside
fn range_score(x: f64, lo: f64, hi: f64, soft: f64) -> f64 {
    if x >= lo && x <= hi {
        1.0
    } else {
        let d = if x < lo { lo - x } else { x - hi };
        (1.0 - d / soft).clamp(0.0, 1.0)
    }
}

/// Sun elevation in degrees for a time and place (NOAA algorithm).
fn solar_elevation_degrees(time: DateTime<FixedOffset>, lat: f64, lon: f64) -> f64 {
    let utc = time.with_timezone(&Utc);
    let unix_seconds = utc.timestamp() as f64 + f64::from(utc.timestamp_subsec_nanos()) / 1e9;
    let julian_day = unix_seconds / 86_400.0 + 2_440_587.5;
    let century = (julian_day - 2_451_545.0) / 36_525.0;

    let mean_longitude =
        (280.46646 + century * (36_000.76983 + century * 0.0003032)).rem_euclid(360.0);
    let mean_anomaly = 357.52911 + century * (35_999.05029 - 0.0001537 * century);
    let eccentricity = 0.016708634 - century * (0.000042037 + 0.0000001267 * century);
    let anomaly_rad = mean_anomaly.to_radians();

    let equation_of_center = anomaly_rad.sin()
        * (1.914602 - century * (0.004817 + 0.000014 * century))
        + (2.0 * anomaly_rad).sin() * (0.019993 - 0.000101 * century)
        + (3.0 * anomaly_rad).sin() * 0.000289;
    let omega = (125.04 - 1934.136 * century).to_radians();
    let apparent_longitude = mean_longitude + equation_of_center - 0.00569 - 0.00478 * omega.sin();

    let mean_obliquity = 23.0
        + (26.0 + (21.448 - century * (46.815 + century * (0.00059 - century * 0.001813))) / 60.0)
            / 60.0;
    let obliquity = mean_obliquity + 0.00256 * omega.cos();
    let declination = (obliquity.to_radians().sin() * apparent_longitude.to_radians().sin()).asin();

    let y = (obliquity.to_radians() / 2.0).tan().powi(2);
    let longitude_rad = mean_longitude.to_radians();
    let equation_of_time = 4.0
        * (y * (2.0 * longitude_rad).sin() - 2.0 * eccentricity * anomaly_rad.sin()
            + 4.0 * eccentricity * y * anomaly_rad.sin() * (2.0 * longitude_rad).cos()
            - 0.5 * y.powi(2) * (4.0 * longitude_rad).sin()
            - 1.25 * eccentricity.powi(2) * (2.0 * anomaly_rad).sin())
        .to_degrees();

    let utc_minutes = f64::from(utc.hour()) * 60.0
        + f64::from(utc.minute())
        + f64::from(utc.second()) / 60.0
        + f64::from(utc.timestamp_subsec_nanos()) / 60e9;
    let solar_minutes = (utc_minutes + equation_of_time + 4.0 * lon).rem_euclid(1440.0);
    let hour_angle = (solar_minutes / 4.0 - 180.0).to_radians();
    let latitude = lat.to_radians();
    let cos_zenith = (latitude.sin() * declination.sin()
        + latitude.cos() * declination.cos() * hour_angle.cos())
    .clamp(-1.0, 1.0);

    90.0 - cos_zenith.acos().to_degrees()
}

// ============================================================================
// 5. TIER 1 + TIER 3: evaluate one hour
// ============================================================================
/// Sun elevation below which we call it night. -0.833 = sunset, -6 = civil twilight,
/// -12 = nautical twilight. -8.33 is the chosen "last usable light".
pub const LIGHT_ELEVATION_DEG: f64 = -8.33;

/// Every score ingredient, 0..1. Returned with each hour so the UI / AI can explain it.
#[derive(Serialize, Clone, Copy)]
pub struct Factors {
    pub energy: f64,
    pub period: f64,
    pub direction: f64,
    pub wind: f64,
    pub clean: f64,
    pub steepness: f64,
    pub cross_sea: f64,
    pub weather: f64,
}

#[derive(Serialize)]
pub struct HourEval {
    pub time: DateTime<FixedOffset>,
    pub surfable: bool,
    pub reasons: Vec<&'static str>,
    pub score: f64,
    pub confidence: f64,
    pub power_kw_m: f64,
    pub factors: Factors,
}

pub fn evaluate(spot: &Spot, h: &Hour, lead_h: f64) -> HourEval {
    let mut why: Vec<&'static str> = Vec::new();
    let hs = if spot.use_wind_waves {
        h.hs
    } else {
        h.sw1.map(|p| p.hs).unwrap_or(h.hs)
    };
    let power = h.power_for(spot);
    let rel = wind_rel(spot, h);

    // --- hard gates (tier 1) ---
    if let Some((a, b)) = spot.season_months {
        let m = h.time.month();
        if m < a || m > b {
            why.push("off season");
        }
    }
    if hs < spot.min_hs {
        why.push("too small");
    }
    if hs > spot.max_hs {
        why.push("too big");
    }
    if h.tp < spot.min_period {
        why.push("period too short");
    }
    if angdiff(h.wave_dir, spot.facing).abs() > spot.window {
        why.push("swell outside window");
    }
    if let Some(min) = spot.min_power_kw_m {
        if power < min {
            why.push("too weak");
        }
    }
    match rel {
        WindRel::Onshore if h.wind_kn > spot.max_onshore_kn => why.push("onshore too strong"),
        WindRel::Offshore if h.wind_kn > spot.max_offshore_kn => why.push("offshore too strong"),
        _ => {}
    }
    if let (Some(dir), Some(min)) = (spot.driver_wind_dir, spot.min_driver_wind_kn) {
        if !(in_arc(h.wind_dir, dir) && h.wind_kn >= min) {
            why.push("no meltemi driver");
        }
    }
    if solar_elevation_degrees(h.time, spot.lat, spot.lon) <= LIGHT_ELEVATION_DEG {
        why.push("nighttime");
    }
    if h.weather_code.map_or(false, |c| c >= 95.0) {
        why.push("thunderstorm"); // any model flagging it counts
    }
    if h.cell_km > 25.0 {
        why.push("no wave data near spot");
    }

    // --- score (tier 3) ---
    let f = factors(spot, h, rel, power);
    let surfable = why.is_empty();
    HourEval {
        time: h.time,
        surfable,
        reasons: why,
        score: if surfable { combine(&f) } else { 0.0 },
        confidence: confidence(h, lead_h),
        power_kw_m: power,
        factors: f,
    }
}

fn factors(spot: &Spot, h: &Hour, rel: WindRel, power: f64) -> Factors {
    // wind-sea spots (meltemi / notos): steep, mixed seas are normal, so skip those factors
    let wind_sea = spot.use_wind_waves;
    let meltemi = spot.driver_wind_dir.is_some();
    let ideal_dir = spot.ideal_swell_dir.map(arc_mid).unwrap_or(spot.facing);
    let swell_dir = if wind_sea {
        h.wave_dir
    } else {
        h.sw1.map_or(h.wave_dir, |p| p.dir)
    };

    let wind = if h.wind_kn <= 4.0 && !meltemi {
        1.0 // glassy
    } else {
        let base = match rel {
            WindRel::Offshore => 1.0,
            WindRel::Cross => 0.6,
            WindRel::Onshore if meltemi => 0.7, // onshore is the point at meltemi spots
            WindRel::Onshore => 0.4 * (1.0 - h.wind_kn / spot.max_onshore_kn).clamp(0.0, 1.0),
        };
        // gusty wind (gust well above mean) is worse
        base * (1.0 - 0.3 * h.gust_kn.map_or(0.0, |g| ramp(g - h.wind_kn, 8.0, 20.0)))
    };

    Factors {
        energy: (power / spot.ideal_power_kw_m.unwrap_or(8.0)).clamp(0.0, 1.0),
        period: range_score(h.tp, spot.ideal_period.0, spot.ideal_period.1, 3.0),
        direction: 1.0 - (angdiff(swell_dir, ideal_dir).abs() / spot.window).clamp(0.0, 1.0),
        wind,
        // how much of the sea is real swell
        clean: if wind_sea {
            1.0
        } else {
            h.sw1.map_or(0.5, |p| (p.hs / h.hs).clamp(0.0, 1.0))
        },
        // deep-water wavelength = 1.56 T^2; steep means mushy
        steepness: if wind_sea {
            1.0
        } else {
            1.0 - ramp(h.hs / (1.56 * h.tp * h.tp), 0.025, 0.06)
        },
        // wind chop running across the swell
        cross_sea: if wind_sea {
            1.0
        } else {
            match (h.ww, h.sw1) {
                (Some(w), Some(s)) => {
                    1.0 - (w.hs / h.hs).clamp(0.0, 1.0)
                        * (angdiff(w.dir, s.dir).abs() / 90.0).clamp(0.0, 1.0)
                }
                _ => 1.0,
            }
        },
        weather: 1.0 - 0.2 * h.precip_mm.map_or(0.0, |p| ramp(p, 2.0, 8.0)),
    }
}

/// Energy multiplies (no energy, no score). Quality is a weighted average; weights sum to 1.
fn combine(f: &Factors) -> f64 {
    let quality = 0.25 * f.period
        + 0.20 * f.direction
        + 0.30 * f.wind
        + 0.10 * f.clean
        + 0.075 * f.steepness
        + 0.075 * f.cross_sea;
    100.0 * f.energy * quality * f.weather
}

/// 0..1: how much to trust this hour. Lead time + model agreement + distance to model cell.
fn confidence(h: &Hour, lead_h: f64) -> f64 {
    let lead = 1.0 - 0.6 * ramp(lead_h, 48.0, 168.0);
    let wind =
        1.0 - 0.5 * ramp(h.wind_spread_kn, 2.0, 8.0) - 0.5 * ramp(h.wind_spread_deg, 30.0, 120.0);
    let waves = h
        .hs_alt
        .map_or(0.8, |a| 1.0 - ramp((a - h.hs).abs() / h.hs, 0.15, 0.6));
    let cell = 1.0 - 0.3 * ramp(h.cell_km, 6.0, 15.0);
    (0.45 * lead + 0.30 * wind + 0.15 * waves + 0.10 * cell).clamp(0.0, 1.0)
}

fn tag(c: f64) -> &'static str {
    if c >= 0.7 {
        "firm"
    } else if c >= 0.5 {
        "likely"
    } else {
        "tentative"
    }
}

pub fn label(score: f64) -> &'static str {
    match score {
        s if s < 35.0 => "surfschool",
        s if s < 55.0 => "surfable",
        s if s < 75.0 => "waves",
        _ => "ON",
    }
}

// ============================================================================
// 6. TIER 2: group surfable hours into sessions
// ============================================================================
#[derive(Serialize)]
pub struct Session {
    pub spot: String,
    pub start: DateTime<FixedOffset>,
    pub end: DateTime<FixedOffset>,
    pub peak_time: DateTime<FixedOffset>,
    pub peak_score: f64,
    pub mean_score: f64,
    pub label: &'static str,
    pub surfable_hours: usize,
    pub confidence: f64,
    pub tag: &'static str,
}

pub fn sessions(spot: &str, evals: &[HourEval], max_gap: usize, min_hours: usize) -> Vec<Session> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for (i, e) in evals.iter().enumerate() {
        if !e.surfable {
            continue;
        }
        let extend = matches!(spans.last(), Some(&(_, end)) if i - end <= max_gap + 1);
        if extend {
            spans.last_mut().unwrap().1 = i;
        } else {
            spans.push((i, i));
        }
    }
    spans
        .into_iter()
        .filter(|(s, e)| e - s + 1 >= min_hours)
        .map(|(s, e)| {
            let slice = &evals[s..=e];
            let on: Vec<&HourEval> = slice.iter().filter(|x| x.surfable).collect();
            let n = on.len() as f64;
            let peak = on.iter().max_by(|a, b| a.score.total_cmp(&b.score)).unwrap();
            let mean = on.iter().map(|x| x.score).sum::<f64>() / n;
            let conf = on.iter().map(|x| x.confidence).sum::<f64>() / n;
            Session {
                spot: spot.to_string(),
                start: slice[0].time,
                end: slice[slice.len() - 1].time,
                peak_time: peak.time,
                peak_score: peak.score,
                mean_score: mean,
                label: label(peak.score),
                surfable_hours: on.len(),
                confidence: conf,
                tag: tag(conf),
            }
        })
        .collect()
}

// ============================================================================
// 7. ENTRY POINT: payload JSON in, result JSON out (wrap this with PyO3 later)
// ============================================================================
#[derive(Serialize)]
pub struct SpotOut {
    pub sessions: Vec<Session>,
    /// How often each reason failed an hour, daylight hours only. Feeds the AI summary.
    pub why_not: HashMap<&'static str, usize>,
    pub sea_c: Option<f64>,
    pub air_c: Option<f64>,
    pub hours: Vec<HourEval>,
}

pub fn run(payload_json: &str) -> anyhow::Result<String> {
    let p: Payload = serde_json::from_str(payload_json)?;
    let mut out: HashMap<String, SpotOut> = HashMap::new();

    for (id, s) in &p.spots {
        let hours = build_hours(s);
        let evals: Vec<HourEval> = hours
            .iter()
            .map(|h| evaluate(&s.config, h, (h.time - p.generated_at).num_hours().max(0) as f64))
            .collect();

        let mut why_not: HashMap<&'static str, usize> = HashMap::new();
        for e in evals.iter().filter(|e| !e.reasons.contains(&"nighttime")) {
            for r in &e.reasons {
                *why_not.entry(*r).or_insert(0) += 1;
            }
        }

        out.insert(
            id.clone(),
            SpotOut {
                sessions: sessions(id, &evals, 1, 2),
                why_not,
                sea_c: hours.first().and_then(|h| h.sea_c),
                air_c: hours.first().and_then(|h| h.air_c),
                hours: evals,
            },
        );
    }
    Ok(serde_json::to_string_pretty(&out)?)
}

// ============================================================================
// 8. TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn test_spot() -> Spot {
        Spot {
            lat: 37.09,
            lon: 21.58,
            facing: 270.0,
            window: 180.0,
            min_hs: 0.5,
            max_hs: 3.0,
            min_period: 4.0,
            ideal_period: (5.0, 8.0),
            offshore_dir: (40.0, 130.0),
            max_onshore_kn: 20.0,
            max_offshore_kn: 20.0,
            use_wind_waves: false,
            ideal_swell_dir: None,
            driver_wind_dir: None,
            min_driver_wind_kn: None,
            season_months: None,
            min_power_kw_m: None,
            ideal_power_kw_m: Some(15.0),
        }
    }

    fn test_hour(time: &str) -> Hour {
        Hour {
            time: DateTime::parse_from_rfc3339(time).unwrap(),
            hs: 1.0,
            tp: 6.0,
            tm10: 5.5,
            wave_dir: 270.0,
            ww: None,
            sw1: Some(Part { hs: 1.0, tm01: 5.0, dir: 270.0 }),
            sw2: None,
            wind_kn: 5.0,
            wind_dir: 90.0,
            gust_kn: None,
            wind_spread_kn: 0.0,
            wind_spread_deg: 0.0,
            precip_mm: None,
            weather_code: None,
            air_c: None,
            sea_c: None,
            hs_alt: None,
            cell_km: 3.0,
        }
    }

    #[test]
    fn evaluate_rejects_nighttime_but_keeps_daylight_hours() {
        let spot = test_spot();
        let night = evaluate(&spot, &test_hour("2026-11-17T17:00:00Z"), 0.0);
        assert!(!night.surfable);
        assert!(night.reasons.contains(&"nighttime"));

        let day = evaluate(&spot, &test_hour("2026-11-17T12:00:00Z"), 0.0);
        assert!(day.surfable, "reasons: {:?}", day.reasons);
        assert!(!day.reasons.contains(&"nighttime"));
    }

    #[test]
    fn power_gate_rejects_weak_swell() {
        let mut spot = test_spot();
        spot.min_power_kw_m = Some(5.0); // test hour has 0.49*1*5 = 2.45 kW/m
        let e = evaluate(&spot, &test_hour("2026-11-17T12:00:00Z"), 0.0);
        assert!(e.reasons.contains(&"too weak"));
    }

    #[test]
    fn thunderstorm_is_a_gate() {
        let mut h = test_hour("2026-11-17T12:00:00Z");
        h.weather_code = Some(95.0);
        let e = evaluate(&test_spot(), &h, 0.0);
        assert!(e.reasons.contains(&"thunderstorm"));
    }

    #[test]
    fn arc_mid_handles_wrap() {
        assert!((arc_mid((235.0, 285.0)) - 260.0).abs() < 1e-9);
        assert!((arc_mid((340.0, 30.0)) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn wind_consensus_averages_across_north() {
        // 350 and 10 degrees at the same speed average to 0, not 180
        let (kn, dir, _, spread) = wind_consensus(&[(10.0, 350.0), (10.0, 10.0)]);
        assert!((kn - 10.0).abs() < 1e-9);
        assert!(angdiff(dir, 0.0).abs() < 1e-6);
        assert!((spread - 10.0).abs() < 1e-6);
    }

    #[test]
    fn more_energy_scores_higher() {
        let spot = test_spot();
        let mut small = test_hour("2026-11-17T12:00:00Z");
        small.sw1 = Some(Part { hs: 0.8, tm01: 5.0, dir: 270.0 });
        let mut big = test_hour("2026-11-17T12:00:00Z");
        big.sw1 = Some(Part { hs: 1.8, tm01: 7.0, dir: 270.0 });
        big.hs = 1.8;
        let (a, b) = (evaluate(&spot, &small, 0.0), evaluate(&spot, &big, 0.0));
        assert!(b.score > a.score);
    }

    #[test]
    fn end_to_end_from_json() {
        let json = r#"{
          "generated_at": "2026-11-17T08:00:00Z",
          "spots": { "t": {
            "config": {"lat":37.09,"lon":21.58,"facing":270,"window":60,"min_hs":0.5,"max_hs":3,
              "min_period":4,"ideal_period":[5,8],"offshore_dir":[40,130],
              "max_onshore_kn":20,"max_offshore_kn":20,"name":"ignored extra key"},
            "cell": {"lat":37.0,"lon":21.5,"dist_km":3.0},
            "waves": [
              {"time":"2026-11-17T10:00:00Z","VHM0":1.4,"VTPK":7.0,"VTM10":6.0,"VMDR":265,
               "VHM0_SW1":1.4,"VTM01_SW1":5.5,"VMDR_SW1":265},
              {"time":"2026-11-17T11:00:00Z","VHM0":1.4,"VTPK":7.0,"VTM10":6.0,"VMDR":265,
               "VHM0_SW1":1.4,"VTM01_SW1":5.5,"VMDR_SW1":265},
              {"time":"2026-11-17T12:00:00Z","VHM0":null,"VTPK":7.0,"VTM10":6.0,"VMDR":265}
            ],
            "wind": {
              "a": [{"time":"2026-11-17T10:00:00Z","wind_kn":5,"wind_dir":90},
                    {"time":"2026-11-17T11:00:00Z","wind_kn":5,"wind_dir":90},
                    {"time":"2026-11-17T12:00:00Z","wind_kn":5,"wind_dir":90}],
              "b": [{"time":"2026-11-17T10:00:00Z","wind_kn":7,"wind_dir":100},
                    {"time":"2026-11-17T11:00:00Z","wind_kn":7,"wind_dir":100}]
            }
          }}
        }"#;
        let out: serde_json::Value = serde_json::from_str(&run(json).unwrap()).unwrap();
        let hours = out["t"]["hours"].as_array().unwrap();
        assert_eq!(hours.len(), 2, "the hour with null VHM0 must be skipped");
        assert_eq!(out["t"]["sessions"].as_array().unwrap().len(), 1);
    }
}