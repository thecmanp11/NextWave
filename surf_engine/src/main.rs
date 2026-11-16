use std::collections::HashMap;
use surf_engine::*;

fn main() -> anyhow::Result<()> {
    let spots: HashMap<String, Spot> =
        serde_json::from_str(&std::fs::read_to_string("spots.json")?)?;
    let forecasts: HashMap<String, Vec<Hour>> =
        serde_json::from_str(&std::fs::read_to_string("forecast.json")?)?;

    for (name, spot) in &spots {
        let Some(hours) = forecasts.get(name) else { continue };
        let evals: Vec<HourEval> = hours.iter().map(|h| evaluate(spot, h)).collect();

        let found = sessions(name, &evals, 1, 2);
        if found.is_empty() {
            // show why the next few hours failed, useful while tuning thresholds
            let sample: Vec<_> = evals.iter().take(6).map(|e| (e.time.to_rfc3339(), &e.reasons)).collect();
            println!("{name}: not surfable. first hours: {sample:?}");
        } else {
            println!("{name}: {}", serde_json::to_string_pretty(&found)?);
        }
    }
    Ok(())
}