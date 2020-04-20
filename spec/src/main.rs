use std::fs;
use std::env;
use anyhow::{Result, anyhow};
use feather_protocol_spec::Protocol;

fn main() -> Result<()> {
    verify()?;
    Ok(())
}

fn verify() -> Result<()> {
    let path = env::args()
        .skip(1)
        .next()
        .ok_or(anyhow!("Specify a file path to verify."))?;
    let file = fs::File::open(&path)?;

    let _: Protocol = ron::de::from_reader(file)
        .map_err(|e| anyhow!("{}", e))?;

    println!("Its all good");

    Ok(())
}