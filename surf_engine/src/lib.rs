use chrono::{DateTime, Datelike, FixedOffset, Timelike, Utc};
use serde::{Deserialize, Serialize};

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
    pub ideal_hs: Option<(f64, f64)>,
    pub driver_wind_dir: Option<(f64, f64)>,
    pub min_driver_wind_kn: Option<f64>,
    pub season_months: Option<(u32, u32)>,
    pub min_power_kw_m: Option<f64>,
    pub ideal_power_kw_m: Option<f64>,
}

#[derive(Deserialize, Clone)]
pub struct Hour {
    pub time: DateTime<FixedOffset>,
    #[serde(rename = "VHM0")]
    pub hs: f64,
    // #[serde(rename = "VTPK")]
    #[serde(rename = "VTM10")]
    pub tp: f64,
    #[serde(rename = "VMDR")]
    pub wave_dir: f64,
    #[serde(rename = "VHM0_SW1")]
    pub hs_sw: Option<f64>,
    pub power_kw_m: f64,
    pub wind_kn: f64,
    pub wind_dir: f64,
}

#[derive(Clone, Copy)]
enum WindRel {
    Offshore,
    Cross,
    Onshore,
}

pub struct HourEval {
    pub time: DateTime<FixedOffset>,
    pub surfable: bool,
    pub reasons: Vec<&'static str>,
    pub score: f64,
}

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
}

// ---------- angle helpers (all directions are "coming from") ----------
fn angdiff(a: f64, b: f64) -> f64 {
    (a - b + 540.0).rem_euclid(360.0) - 180.0
}

// true if d lies in the arc from..to, handling wrap through 0
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

fn wind_rel(spot: &Spot, h: &Hour) -> WindRel {
    if in_arc(h.wind_dir, spot.offshore_dir) {
        WindRel::Offshore
    } else if angdiff(h.wind_dir, spot.facing).abs() <= 60.0 {
        WindRel::Onshore
    } else {
        WindRel::Cross
    }
}

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

// ---------- tier 1 + tier 3, per hour ----------
pub fn evaluate(spot: &Spot, h: &Hour) -> HourEval {
    let mut why: Vec<&'static str> = Vec::new();
    let hs = if spot.use_wind_waves {
        h.hs
    } else {
        h.hs_sw.unwrap_or(h.hs)
    };

    // LAST LIGHT: -0.833 degrees is sunrise/set, -6 is "civil twilight"
    if solar_elevation_degrees(h.time, spot.lat, spot.lon) <= -8.33 {
        why.push("nighttime");
    }
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

    let rel = wind_rel(spot, h);
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

    let surfable = why.is_empty();
    let score = if surfable {
        score(spot, h, hs, rel)
    } else {
        0.0
    };
    HourEval {
        time: h.time,
        surfable,
        reasons: why,
        score,
    }
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

// fn score(spot: &Spot, h: &Hour, hs: f64, rel: WindRel) -> f64 {
//     let (lo, hi) = spot.ideal_hs.unwrap_or((spot.min_hs * 1.5, spot.max_hs * 0.7));
//     let size = range_score(hs, lo, hi, (hi - lo).max(0.5));
//     let period = range_score(h.tp, spot.ideal_period.0, spot.ideal_period.1, 3.0);
//     let dir = 1.0 - (angdiff(h.wave_dir, spot.facing).abs() / spot.window).clamp(0.0, 1.0);
//     let wind = match rel {
//         WindRel::Offshore => 1.0,
//         WindRel::Cross => 0.6,
//         WindRel::Onshore => {
//             if spot.driver_wind_dir.is_some() {
//                 0.7 // meltemi spots: onshore is the point
//             } else {
//                 0.4 * (1.0 - h.wind_kn / spot.max_onshore_kn).clamp(0.0, 1.0)
//             }
//         }
//     };
//     let clean = if spot.use_wind_waves {
//         1.0
//     } else {
//         h.hs_sw.map(|s| (s / h.hs).clamp(0.0, 1.0)).unwrap_or(0.5)
//     };
//     100.0 * (0.30 * size + 0.20 * period + 0.15 * dir + 0.25 * wind + 0.10 * clean)
// }

fn score(spot: &Spot, h: &Hour, _hs: f64, rel: WindRel) -> f64 {
    let ideal = spot.ideal_power_kw_m.unwrap_or(8.0);
    let energy = (h.power_kw_m / ideal).clamp(0.0, 1.0); // no energy, no score

    let period = range_score(h.tp, spot.ideal_period.0, spot.ideal_period.1, 3.0);
    let dir = 1.0 - (angdiff(h.wave_dir, spot.facing).abs() / spot.window).clamp(0.0, 1.0);
    let wind = match rel {
        WindRel::Offshore => 1.0,
        WindRel::Cross => 0.6,
        WindRel::Onshore => {
            if spot.driver_wind_dir.is_some() {
                0.7
            } else {
                0.4 * (1.0 - h.wind_kn / spot.max_onshore_kn).clamp(0.0, 1.0)
            }
        }
    };
    let clean = if spot.use_wind_waves {
        1.0
    } else {
        h.hs_sw.map(|s| (s / h.hs).clamp(0.0, 1.0)).unwrap_or(0.5)
    };

    let quality = 0.30 * period + 0.35 * wind + 0.20 * dir + 0.15 * clean;
    100.0 * energy * quality
}

pub fn label(score: f64) -> &'static str {
    match score {
        s if s < 35.0 => "surfschool",
        s if s < 55.0 => "surfable",
        s if s < 75.0 => "waves",
        _ => "ON",
    }
}

// ---------- tier 2: group hours into sessions ----------
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
            let peak = on
                .iter()
                .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
                .unwrap();
            let mean = on.iter().map(|x| x.score).sum::<f64>() / on.len() as f64;
            Session {
                spot: spot.to_string(),
                start: slice[0].time,
                end: slice[slice.len() - 1].time,
                peak_time: peak.time,
                peak_score: peak.score,
                mean_score: mean,
                label: label(peak.score),
                surfable_hours: on.len(),
            }
        })
        .collect()
}

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
            ideal_hs: None,
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
            wave_dir: 270.0,
            hs_sw: Some(1.0),
            power_kw_m: 10.0,
            wind_kn: 5.0,
            wind_dir: 90.0,
        }
    }

    #[test]
    fn evaluate_rejects_nighttime_but_keeps_daylight_hours() {
        let spot = test_spot();

        let night = evaluate(&spot, &test_hour("2026-11-17T16:00:00Z"));
        assert!(!night.surfable);
        assert!(night.reasons.contains(&"nighttime"));

        let day = evaluate(&spot, &test_hour("2026-11-17T12:00:00Z"));
        assert!(day.surfable);
        assert!(!day.reasons.contains(&"nighttime"));
    }
}
