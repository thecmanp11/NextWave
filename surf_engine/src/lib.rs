use chrono::{DateTime, Datelike, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct Spot {
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
}

#[derive(Deserialize, Clone)]
pub struct Hour {
    pub time: DateTime<FixedOffset>,
    #[serde(rename = "VHM0")]
    pub hs: f64,
    #[serde(rename = "VTPK")]
    pub tp: f64,
    #[serde(rename = "VMDR")]
    pub wave_dir: f64,
    #[serde(rename = "VHM0_SW1")]
    pub hs_sw: Option<f64>,
    pub wind_kn: f64,
    pub wind_dir: f64,
}

#[derive(Clone, Copy)]
enum WindRel { Offshore, Cross, Onshore }

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
    let (d, from, to) = (d.rem_euclid(360.0), from.rem_euclid(360.0), to.rem_euclid(360.0));
    if from <= to { d >= from && d <= to } else { d >= from || d <= to }
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

// ---------- tier 1 + tier 3, per hour ----------
pub fn evaluate(spot: &Spot, h: &Hour) -> HourEval {
    let mut why: Vec<&'static str> = Vec::new();
    let hs = if spot.use_wind_waves { h.hs } else { h.hs_sw.unwrap_or(h.hs) };

    if let Some((a, b)) = spot.season_months {
        let m = h.time.month();
        if m < a || m > b { why.push("off season"); }
    }
    if hs < spot.min_hs { why.push("too small"); }
    if hs > spot.max_hs { why.push("too big"); }
    if h.tp < spot.min_period { why.push("period too short"); }
    if angdiff(h.wave_dir, spot.facing).abs() > spot.window { why.push("swell outside window"); }

    let rel = wind_rel(spot, h);
    match rel {
        WindRel::Onshore if h.wind_kn > spot.max_onshore_kn => why.push("onshore too strong"),
        WindRel::Offshore if h.wind_kn > spot.max_offshore_kn => why.push("offshore too strong"),
        _ => {}
    }
    if let (Some(dir), Some(min)) = (spot.driver_wind_dir, spot.min_driver_wind_kn) {
        if !(in_arc(h.wind_dir, dir) && h.wind_kn >= min) { why.push("no meltemi driver"); }
    }

    let surfable = why.is_empty();
    let score = if surfable { score(spot, h, hs, rel) } else { 0.0 };
    HourEval { time: h.time, surfable, reasons: why, score }
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

fn score(spot: &Spot, h: &Hour, hs: f64, rel: WindRel) -> f64 {
    let (lo, hi) = spot.ideal_hs.unwrap_or((spot.min_hs * 1.5, spot.max_hs * 0.7));
    let size = range_score(hs, lo, hi, (hi - lo).max(0.5));
    let period = range_score(h.tp, spot.ideal_period.0, spot.ideal_period.1, 3.0);
    let dir = 1.0 - (angdiff(h.wave_dir, spot.facing).abs() / spot.window).clamp(0.0, 1.0);
    let wind = match rel {
        WindRel::Offshore => 1.0,
        WindRel::Cross => 0.6,
        WindRel::Onshore => {
            if spot.driver_wind_dir.is_some() {
                0.7 // meltemi spots: onshore is the point
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
    100.0 * (0.30 * size + 0.20 * period + 0.15 * dir + 0.25 * wind + 0.10 * clean)
}

pub fn label(score: f64) -> &'static str {
    match score {
        s if s < 35.0 => "poor",
        s if s < 55.0 => "fair",
        s if s < 75.0 => "good",
        _ => "excellent",
    }
}

// ---------- tier 2: group hours into sessions ----------
pub fn sessions(spot: &str, evals: &[HourEval], max_gap: usize, min_hours: usize) -> Vec<Session> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for (i, e) in evals.iter().enumerate() {
        if !e.surfable { continue; }
        let extend = matches!(spans.last(), Some(&(_, end)) if i - end <= max_gap + 1);
        if extend { spans.last_mut().unwrap().1 = i; } else { spans.push((i, i)); }
    }
    spans
        .into_iter()
        .filter(|(s, e)| e - s + 1 >= min_hours)
        .map(|(s, e)| {
            let slice = &evals[s..=e];
            let on: Vec<&HourEval> = slice.iter().filter(|x| x.surfable).collect();
            let peak = on.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap()).unwrap();
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