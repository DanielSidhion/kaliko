use network::headers::BlockHeader;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::Result;

pub struct BlockHeaderStorage {
    storage_file: File,
    header_count: u32,
    pub latest_header: BlockHeader,
}

impl BlockHeaderStorage {
    pub fn new(storage_location: &str) -> BlockHeaderStorage {
        let mut storage_file = OpenOptions::new().read(true).append(true).create(true).open(storage_location).unwrap();

        let mut latest_header = BlockHeader::new_genesis();
        let mut header_count = 0;
        while let Ok(header) = BlockHeader::deserialize(&mut storage_file) {
            latest_header = header;
            header_count += 1;
        }

        if header_count == 0 {
            latest_header.serialize(&mut storage_file).unwrap();
            header_count += 1;
        }

        BlockHeaderStorage {
            storage_file,
            header_count,
            latest_header,
        }
    }

    pub fn num_headers(&self) -> u32 {
        self.header_count
    }

    pub fn write_headers(&mut self, headers: &Vec<BlockHeader>) -> Result<()> {
        self.latest_header = headers.last().unwrap().clone();

        for header in headers {
            // TODO: convert errors here or convert std::io errors to NetworkError (and change it to be wider name)?
            header.serialize(&mut self.storage_file).unwrap();
            self.header_count += 1;
        }

        self.storage_file.flush()?;
        Ok(())
    }
}