use std::fs;
use std::env;
use anyhow::{Result, anyhow};
use feather_protocol_spec::{Protocol, Validation};

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

    let protocol: Protocol = ron::de::from_reader(file)
        .map_err(|e| anyhow!("{}", e))?;

    protocol.validate()?;

    Ok(())
}