use std::fs;
use std::env;
use anyhow::{Result, anyhow};
use feather_protocol_spec::{Protocol, Validation};
use std::io::{Write, Seek, SeekFrom};

fn main() -> Result<()> {
    verify()?;
    Ok(())
}

fn verify() -> Result<()> {
    let path = env::args()
        .skip(1)
        .next()
        .ok_or(anyhow!("Specify a file path to verify."))?;
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(false)
        .open(&path)?;

    let protocol: Protocol = ron::de::from_reader(&file)
        .map_err(|e| anyhow!("{}", e))?;

    protocol.validate()?;

    let protocol_ser = ron::ser::to_string_pretty(&protocol, Default::default())?;
    
    file.seek(SeekFrom::Start(0))?;
    let buf = protocol_ser.as_bytes();

    file.write_all(buf)?;
    file.set_len(buf.len() as u64)?;

    Ok(())
}