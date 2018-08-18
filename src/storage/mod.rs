use std::fs::{File, OpenOptions};
use std::io::prelude::*;

pub struct BlockHeaderStorage {
    storage_file: File,
}

impl BlockHeaderStorage {
    pub fn new(storage_location: &str) -> BlockHeaderStorage {
        let storage_file = OpenOptions::new().read(true).append(true).create(true).open(storage_location).unwrap();

        BlockHeaderStorage {
            storage_file,
        }
    }

    pub fn write_headers()
}