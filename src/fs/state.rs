use std::io::{Error, BufReader};
use serde::{Serialize, Deserialize};
use hex::ToHex;
use super::hash_reader;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct Identity {
    hash: String,
    size: u32
}

impl Identity {
    pub fn new(hash: String, size: u32) -> Self {
        Self { hash, size }
    }

    pub fn size(&self) -> u32 { self.size }
    pub fn hash(&self) -> String { self.hash.clone() }
}

impl TryFrom<String> for Identity {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let reader = BufReader::new(std::fs::File::open(&value)?);
        let digest = hash_reader(reader)?;
        let size = std::fs::metadata(&value)?.len() as u32;

        Ok(Identity {
            hash: digest.encode_hex(),
            size
        })
    }
}


#[derive(Debug)]
pub struct Local { pub size: u64 }
#[derive(PartialEq, Debug)]
pub struct LocalHashed { pub local: Identity }
#[derive(PartialEq, Debug)]
pub struct Remote { pub remote: Identity }
#[derive(PartialEq, Debug)]
pub struct Sync { pub id: Identity }
#[derive(PartialEq, Debug)]
pub struct Diff { pub local: Identity, pub remote: Identity }

impl LocalHashed {
    pub fn new(id: Identity) -> Self {
        Self { local: id }
    }
}

impl Local {
    pub fn new(size: u64) -> Self {
        Self { size }
    }
}
