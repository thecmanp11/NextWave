use std::collections::BTreeMap;
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

// TWO hour max gap for bare min surfability until time range is broken up
pub const MAX_GAP: usize = 2;
// civil twilight is -6, REAL night night is -12
pub const LAST_LIGHT_DEG: f64 = -8.0;
pub const DEEP_WATER_POWER_COEFF: f64 = 0.49;
pub const GRAVITY: f64 = 9.80665;

#[derive(Deserialize, Clone, Debug)]
pub struct WaveSystem {
    pub hs: Option<f64>,
    pub period: Option<f64>,
    pub direction: Option<f64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Hour {
    pub time: DateTime<Utc>,
    pub depth_m: Option<f64>,
    pub total: WaveSystem,
    pub wind_sea: WaveSystem,
    pub primary_swell: WaveSystem,
    pub secondary_swell: WaveSystem,
    pub wind_kn: f64,
    pub wind_gust_kn: Option<f64>,
    pub wind_dir: f64,
}

#[derive(Deserialize, Clone, Debug)]
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
    pub max_crossshore_kn: f64,
    #[serde(default)]
    pub ideal_swell_dir: Option<(f64, f64)>,
    #[serde(default)]
    pub ideal_hs: Option<(f64, f64)>,
    pub min_power_kw_m: Option<f64>,
    pub ideal_power_kw_m: Option<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WindRel {
    Offshore,
    Cross,
    Onshore,
}

#[derive(Debug)]
struct WaveDecision {
    ok: bool,
    reasons: Vec<&'static str>,
    hs: f64,
    period: f64,
    direction: f64,
    power_kw_m: f64,
    clean_ratio: f64,
}

#[derive(Debug)]
struct WindDecision {
    ok: bool,
    reasons: Vec<&'static str>,
    rel: WindRel,
    quality: f64,
}

#[derive(Debug)]
struct ContextDecision {
    ok: bool,
    reasons: Vec<&'static str>,
}

#[derive(Serialize, Clone, Debug)]
pub struct HourEval {
    pub time: DateTime<Utc>,
    pub surfable: bool,
    pub reasons: Vec<&'static str>,
    pub score: f64,
    pub power_kw_m: f64,
}

// data output
#[derive(Serialize, Clone, Debug)]
pub struct Session {
    pub spot: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub peak_time: DateTime<Utc>,
    pub peak_score: f64,
    pub mean_score: f64,
    pub surfable_hours: usize,
    pub hourly_scores: BTreeMap<String, f64>,
}

// ==================== SHARED GEOMETRY & MATH ====================
fn angdiff(a: f64, b: f64) -> f64 {
    (a - b + 540.0).rem_euclid(360.0) - 180.0
}

fn in_arc(d: f64, (from, to): (f64, f64)) -> bool {
    let (d, from, to) = (
        d.rem_euclid(360.0),
        from.rem_euclid(360.0),
        to.rem_euclid(360.0),
    );
    if from <= to {
        d >= from && d <= to
    } else {
        d >= from || d <= to
    }
}

fn range_score(x: f64, lo: f64, hi: f64, soft: f64) -> f64 {
    if x >= lo && x <= hi {
        1.0
    } else {
        let d = if x < lo { lo - x } else { x - hi };
        (1.0 - d / soft.max(1e-6)).clamp(0.0, 1.0)
    }
}

fn dir_alignment_score(spot: &Spot, wave_dir: f64) -> f64 {
    let window_q = (1.0 - (angdiff(wave_dir, spot.facing).abs() / spot.window.max(1.0)))
        .clamp(0.0, 1.0);
    if let Some((lo, hi)) = spot.ideal_swell_dir {
        let lo_n = lo.rem_euclid(360.0);
        let hi_n = hi.rem_euclid(360.0);
        let mid = if lo_n <= hi_n {
            0.5 * (lo_n + hi_n)
        } else {
            (0.5 * (lo_n + hi_n + 360.0)).rem_euclid(360.0)
        };
        let half_span = (angdiff(hi_n, lo_n).abs() * 0.5).max(10.0);
        let ideal_dist = angdiff(wave_dir, mid).abs();
        let ideal_q = if ideal_dist <= half_span {
            1.0
        } else {
            (1.0 - (ideal_dist - half_span) / 30.0).clamp(0.0, 1.0)
        };
        0.5 * window_q + 0.5 * ideal_q
    } else {
        window_q
    }
}

/// Period-aware angular diffraction/spread margin (in degrees) beyond `spot.window`.
fn effective_window_deg(spot: &Spot, period_s: f64) -> f64 {
    let period_bonus = ((period_s - 4.0) * 1.5).clamp(0.0, 8.0);
    spot.window + 5.0 + period_bonus
}

/// Ratio of finite-depth wave group velocity `c_g(h)` to deep-water group velocity `c_g0`.
/// Returns `1.0` when depth is unknown or deep (`h / L_0 >= 0.5`).
pub fn finite_depth_power_factor(period_s: f64, depth_m: Option<f64>) -> f64 {
    let Some(h) = depth_m else {
        return 1.0;
    };
    if !(h > 0.0 && period_s > 0.0) {
        return 1.0;
    }

    let omega = std::f64::consts::TAU / period_s;
    let l0 = GRAVITY * period_s.powi(2) / std::f64::consts::TAU;
    if h / l0 >= 0.5 {
        return 1.0;
    }

    // Solve (omega^2 * h / g) = x * tanh(x) for x = k * h
    let target = omega.powi(2) * h / GRAVITY;
    let mut x = target.sqrt().max(1e-4);
    for _ in 0..8 {
        let th = x.tanh();
        let sech2 = 1.0 - th * th;
        let f = x * th - target;
        let df = th + x * sech2;
        if df.abs() < 1e-12 {
            break;
        }
        x = (x - f / df).max(1e-6);
    }

    let th = x.tanh().clamp(1e-6, 1.0);
    let sinh2x = (2.0 * x).sinh();
    let n = if sinh2x.is_finite() && sinh2x > 1e-9 {
        0.5 * (1.0 + (2.0 * x) / sinh2x)
    } else {
        1.0
    };

    // c_g / c_g0 = 2 * n * tanh(k * h)
    (2.0 * n * th).clamp(0.15, 1.25)
}

pub fn wave_power_kw_m(hs: Option<f64>, period_s: Option<f64>, depth_m: Option<f64>) -> f64 {
    match (hs, period_s) {
        (Some(h_s), Some(t)) if h_s > 0.0 && t > 0.0 => {
            DEEP_WATER_POWER_COEFF * h_s.powi(2) * t * finite_depth_power_factor(t, depth_m)
        }
        _ => 0.0,
    }
}

impl WaveSystem {
    pub fn power_kw_m(&self, depth_m: Option<f64>) -> f64 {
        wave_power_kw_m(self.hs, self.period, depth_m)
    }
}


// ==================== DECISION GROUP 1: WAVES ====================
fn assess_waves(spot: &Spot, h: &Hour) -> WaveDecision {
    let mut reasons = Vec::new();
    let total_hs = h.total.hs.unwrap_or(0.0).max(0.0);
    let total_t = h.total.period.unwrap_or(0.0).max(0.0);
    let total_dir = h.total.direction.unwrap_or(spot.facing);

    let sw1_hs = h.primary_swell.hs.unwrap_or(0.0).max(0.0);
    let sw1_t = h.primary_swell.period.unwrap_or(total_t).max(0.0);
    let sw1_dir = h.primary_swell.direction.unwrap_or(total_dir);

    let sw2_hs = h.secondary_swell.hs.unwrap_or(0.0).max(0.0);
    let sw2_t = h.secondary_swell.period.unwrap_or(sw1_t).max(0.0);
    let sw2_dir = h.secondary_swell.direction.unwrap_or(sw1_dir);

    let combined_swell_hs = sw1_hs + sw2_hs;
    let ww_hs = h
        .wind_sea
        .hs
        .unwrap_or((total_hs - combined_swell_hs).max(0.0))
        .max(0.0);
    let ww_t = h.wind_sea.period.unwrap_or(total_t).max(0.0);
    let ww_dir = h.wind_sea.direction.unwrap_or(total_dir);

    // Spectral energy proxies E ~ H^2 * T across partitions
    let e_sw1 = if sw1_hs > 0.0 && sw1_t > 0.0 { sw1_hs.powi(2) * sw1_t } else { 0.0 };
    let e_sw2 = if sw2_hs > 0.0 && sw2_t > 0.0 { sw2_hs.powi(2) * sw2_t } else { 0.0 };
    let e_ww = if ww_hs > 0.0 && ww_t > 0.0 { ww_hs.powi(2) * ww_t } else { 0.0 };
    let e_parts = e_sw1 + e_sw2 + e_ww;

    // Variance-additive effective wave height (m_0 ~ H^2 / 16), capped at total_hs when known
    let part_var_hs = (sw1_hs.powi(2) + sw2_hs.powi(2) + ww_hs.powi(2)).sqrt();
    let hs = if total_hs > 0.0 && part_var_hs > 0.0 {
        part_var_hs.min(total_hs)
    } else if part_var_hs > 0.0 {
        part_var_hs
    } else {
        total_hs
    };

    // Energy-weighted wave period across available partitions, with fallback to total/wind-sea period
    let period = if e_parts > 1e-6 {
        (e_sw1 * sw1_t + e_sw2 * sw2_t + e_ww * ww_t) / e_parts
    } else if total_t > 0.0 {
        total_t
    } else {
        ww_t.max(sw1_t).max(sw2_t)
    };

    // Dominant wave direction by energy (E ~ H^2 * T)
    let mut best_e = 0.0;
    let mut direction = total_dir;
    for (e, d) in [(e_sw1, sw1_dir), (e_sw2, sw2_dir), (e_ww, ww_dir)] {
        if e > best_e {
            best_e = e;
            direction = d;
        }
    }
    if best_e <= 1e-6 && h.total.direction.is_none() {
        direction = spot.facing;
    }

    let power_kw_m = if hs > 0.0 && period > 0.0 {
        wave_power_kw_m(Some(hs), Some(period), h.depth_m)
    } else {
        h.total.power_kw_m(h.depth_m)
    };

    if hs < spot.min_hs {
        reasons.push("too small");
    }
    if hs > spot.max_hs {
        reasons.push("too big");
    }
    if period < spot.min_period {
        reasons.push("period too short");
    }
    let max_angle = effective_window_deg(spot, period);
    if angdiff(direction, spot.facing).abs() > max_angle {
        reasons.push("swell outside window");
    }
    if let Some(min_p) = spot.min_power_kw_m {
        if power_kw_m < min_p {
            reasons.push("too weak");
        }
    }

    // Swell-to-total energy fraction (0 = pure wind sea, 1 = pure swell)
    let swell_energy_share = if e_parts > 1e-6 {
        ((e_sw1 + e_sw2) / e_parts).clamp(0.0, 1.0)
    } else if total_hs > 1e-6 {
        (combined_swell_hs / total_hs).clamp(0.0, 1.0)
    } else {
        0.5
    };
    let clean_ratio = (0.35 + 0.65 * swell_energy_share).clamp(0.0, 1.0);

    WaveDecision {
        ok: reasons.is_empty(),
        reasons,
        hs,
        period,
        direction,
        power_kw_m,
        clean_ratio,
    }
}

// ==================== DECISION GROUP 2: WIND ====================
fn assess_wind(spot: &Spot, h: &Hour, hs: f64) -> WindDecision {
    let mut reasons = Vec::new();
    let rel = if in_arc(h.wind_dir, spot.offshore_dir) {
        WindRel::Offshore
    } else if angdiff(h.wind_dir, spot.facing).abs() <= 60.0 {
        WindRel::Onshore
    } else {
        WindRel::Cross
    };

    let peak_kn = h.wind_gust_kn.unwrap_or(h.wind_kn).max(h.wind_kn);
    match rel {
        WindRel::Onshore if h.wind_kn > spot.max_onshore_kn => reasons.push("onshore too strong"),
        WindRel::Offshore if peak_kn > spot.max_offshore_kn => reasons.push("offshore too strong"),
        WindRel::Cross if h.wind_kn > spot.max_crossshore_kn => reasons.push("crossshore too strong"),
        _ => {}
    }

    let quality = match rel {
        WindRel::Offshore => {
            let over_excess = (peak_kn - 14.0).max(0.0) / 14.0;
            let small_wave_chop = if hs < 1.0 && h.wind_kn > 12.0 {
                ((h.wind_kn - 12.0) / 15.0).clamp(0.0, 0.35)
            } else {
                0.0
            };
            (1.0 - 0.45 * over_excess - small_wave_chop).clamp(0.35, 1.0)
        }
        WindRel::Cross => {
            let cap = spot.max_crossshore_kn.max(1.0);
            let speed_damp = (1.0 - (h.wind_kn / cap) * 0.45).clamp(0.40, 1.0);
            0.68 * speed_damp
        }
        WindRel::Onshore => {
            let cap = spot.max_onshore_kn.max(1.0);
            (0.45 * (1.0 - h.wind_kn / cap)).clamp(0.05, 0.45)
        }
    };

    WindDecision {
        ok: reasons.is_empty(),
        reasons,
        rel,
        quality,
    }
}

// ==================== DECISION GROUP 3: CONTEXT (LIGHT & SEASON) ====================
fn solar_elevation_degrees(time: DateTime<Utc>, lat: f64, lon: f64) -> f64 {
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
    let apparent_longitude =
        mean_longitude + equation_of_center - 0.00569 - 0.00478 * omega.sin();

    let mean_obliquity = 23.0
        + (26.0
            + (21.448 - century * (46.815 + century * (0.00059 - century * 0.001813))) / 60.0)
            / 60.0;
    let obliquity = mean_obliquity + 0.00256 * omega.cos();
    let declination =
        (obliquity.to_radians().sin() * apparent_longitude.to_radians().sin()).asin();

    let y = (obliquity.to_radians() / 2.0).tan().powi(2);
    let longitude_rad = mean_longitude.to_radians();
    let equation_of_time = 4.0
        * (y * (2.0 * longitude_rad).sin()
            - 2.0 * eccentricity * anomaly_rad.sin()
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

fn assess_context(spot: &Spot, h: &Hour) -> ContextDecision {
    let mut reasons = Vec::new();
    if solar_elevation_degrees(h.time, spot.lat, spot.lon) <= LAST_LIGHT_DEG {
        reasons.push("too dark");
    }
    ContextDecision {
        ok: reasons.is_empty(),
        reasons,
    }
}

// ==================== FINAL DECISION & SCORING ====================
fn final_score(spot: &Spot, w: &WaveDecision, wi: &WindDecision) -> f64 {
    let ideal_p = spot.ideal_power_kw_m.unwrap_or(8.0).max(1.0);
    let floor_p = spot.min_power_kw_m.unwrap_or(5.0).min(ideal_p * 0.85);

    // Starts above 0 for bare-minimum rideable waves and ramps to 1.0 at ideal power
    let energy = if w.power_kw_m <= floor_p {
        0.18
    } else {
        (0.18 + 0.82 * ((w.power_kw_m - floor_p) / (ideal_p - floor_p))).clamp(0.18, 1.0)
    };

    let hs_factor = spot
        .ideal_hs
        .map(|(lo, hi)| 0.8 + 0.2 * range_score(w.hs, lo, hi, 1.0))
        .unwrap_or(1.0);

    let period_q = range_score(w.period, spot.ideal_period.0, spot.ideal_period.1, 3.0);
    let dir_q = dir_alignment_score(spot, w.direction);

    let quality = 0.30 * period_q + 0.35 * wi.quality + 0.20 * dir_q + 0.15 * w.clean_ratio;
    (100.0 * energy * hs_factor * quality).clamp(1.0, 100.0)
}

pub fn evaluate(spot: &Spot, h: &Hour) -> HourEval {
    let w = assess_waves(spot, h);
    let wi = assess_wind(spot, h, w.hs);
    let ctx = assess_context(spot, h);

    let mut reasons = Vec::new();
    reasons.extend(ctx.reasons);
    reasons.extend(w.reasons.iter().copied());
    reasons.extend(wi.reasons.iter().copied());

    let surfable = w.ok && wi.ok && ctx.ok;
    let score = if surfable {
        final_score(spot, &w, &wi)
    } else {
        0.0
    };

    HourEval {
        time: h.time,
        surfable,
        reasons,
        score,
        power_kw_m: w.power_kw_m,
    }
}

// return data
pub fn sessions(spot: &str, evals: &[HourEval], min_hours: usize) -> Vec<Session> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for (i, e) in evals.iter().enumerate() {
        if !e.surfable {
            continue;
        }
        let extend = matches!(spans.last(), Some(&(_, end)) if i - end <= MAX_GAP + 1);
        if extend {
            spans.last_mut().unwrap().1 = i;
        } else {
            spans.push((i, i));
        }
    }

    spans
        .into_iter()
        .filter_map(|(s, e)| {
            let slice = &evals[s..=e];
            let on: Vec<&HourEval> = slice.iter().filter(|x| x.surfable).collect();
            if on.len() < min_hours {
                return None;
            }
            // let peak = on.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())?;
            let peak = on.iter().max_by(|a, b| a.score.total_cmp(&b.score))?;
            let mean = on.iter().map(|x| x.score).sum::<f64>() / on.len() as f64;

            // scores per surfable hour
            let hourly_scores: BTreeMap<String, f64> = on
                .iter()
                .map(|x| (x.time.to_rfc3339(), x.score))
                .collect();

            Some(Session {
                spot: spot.to_string(),
                start: on.first()?.time,
                end: on.last()?.time,
                peak_time: peak.time,
                peak_score: peak.score,
                mean_score: mean,
                surfable_hours: on.len(),
                hourly_scores,
            })
        })
        .collect()
}