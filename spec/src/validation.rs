use crate::*;

use anyhow::{bail, Result};
use std::collections::BTreeMap;

pub trait Validation {
    fn validate(&self) -> Result<()>;
}

impl Validation for Protocol {
    fn validate(&self) -> Result<()> {
        let mut packet_names = BTreeMap::new();
        let mut prev: Option<&PacketIdentifier> = None;
        for (identifier, packet) in self.packets.iter() {
            if let Some(prev_identifier) = prev {
                if prev_identifier.direction() == identifier.direction()
                    && prev_identifier.stage() == identifier.stage()
                    && !(*prev_identifier.id() + 1 == *identifier.id()) {
                    println!("Skiped from {:#X?} to {:#X?}", prev_identifier, identifier);
                }
            }
            prev = Some(identifier);
            if let Some(old_identifier) = packet_names.insert(
                (
                    identifier.direction(),
                    identifier.stage(),
                    packet.name()
                ),
                identifier,
            ) {
                bail!(
                    "The packet name \"{}\" for {:#X?} is already used by {:#X?}.",
                    packet.name(),
                    identifier,
                    old_identifier
                );
            }
        }
        Ok(())
    }
}
