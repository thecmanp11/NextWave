use std::collections::HashMap;
use surf_engine::*;

fn main() -> anyhow::Result<()> {
    let spots: HashMap<String, Spot> =
        serde_json::from_str(&std::fs::read_to_string("spots.json")?)?;
    let forecasts: HashMap<String, Vec<Hour>> =
        serde_json::from_str(&std::fs::read_to_string("forecast.json")?)?;

    let mut all_sessions: Vec<Session> = Vec::new();

    for (name, spot) in &spots {
        let Some(hours) = forecasts.get(name) else { continue };
        let evals: Vec<HourEval> = hours.iter().map(|h| evaluate(spot, h)).collect();

        let found = sessions(name, &evals, 1, 2);
        if found.is_empty() {
            // Use eprintln! so it prints to stderr and leaves stdout completely pure for JSON
            let sample: Vec<_> = evals.iter().take(6).map(|e| (e.time.to_rfc3339(), &e.reasons)).collect();
            eprintln!("{name}: not surfable. first hours: {sample:?}");
        } else {
            eprintln!("{name}: found {} session(s)", found.len());
            all_sessions.extend(found);
        }
    }

    // THIS IS THE ONLY THING PRINTED TO STDOUT:
    // A single, clean JSON array of all sessions across all spots
    let final_json = serde_json::to_string(&all_sessions)?;
    println!("{}", final_json);

    Ok(())
}