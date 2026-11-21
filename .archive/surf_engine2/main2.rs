fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "payload.json".into());
    println!("{}", surf_engine::run(&std::fs::read_to_string(&path)?)?);
    Ok(())
}