use std::collections::BTreeMap;
use std::fs;
use surf_engine::{evaluate, sessions, Hour, HourEval, Spot};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let forecast_path = args.get(1).map(String::as_str).unwrap_or("forecast.json");

    let spots: BTreeMap<String, Spot> = serde_json::from_str(&fs::read_to_string("spots.json")?)?;
    let forecast: BTreeMap<String, Vec<Hour>> =
        serde_json::from_str(&fs::read_to_string(forecast_path)?)?;

    let mut output = BTreeMap::new();
    for (name, hours) in &forecast {
        let Some(spot) = spots.get(name) else { continue };
        let evals: Vec<HourEval> = hours.iter().map(|h| evaluate(spot, h)).collect();
        let sess = sessions(name, &evals, 2);
        output.insert(name.clone(), sess);
    }

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}