use ::KalikoControlMessage;
use network::headers::BlockHeader;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::Result;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub struct BlockHeaderStorage {
    storage_file: File,
    header_count: u32,
    pub latest_header: BlockHeader,
    incoming_control_sender: Sender<KalikoControlMessage>,
    incoming_control_receiver: Receiver<KalikoControlMessage>,
    outgoing_control_sender: Sender<KalikoControlMessage>,
}

impl BlockHeaderStorage {
    pub fn new(storage_location: &str, outgoing_control_sender: Sender<KalikoControlMessage>) -> BlockHeaderStorage {
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

        let (incoming_control_sender, incoming_control_receiver) = channel();

        BlockHeaderStorage {
            storage_file,
            header_count,
            latest_header,
            incoming_control_sender,
            incoming_control_receiver,
            outgoing_control_sender,
        }
    }

    pub fn num_headers(&self) -> u32 {
        self.header_count
    }

    pub fn incoming_sender(&self) -> Sender<KalikoControlMessage> {
        self.incoming_control_sender.clone()
    }

    fn write_headers(&mut self, headers: &Vec<BlockHeader>) -> Result<()> {
        self.latest_header = headers.last().unwrap().clone();

        for header in headers {
            // TODO: convert errors here or convert std::io errors to NetworkError (and change it to be wider name)?
            header.serialize(&mut self.storage_file).unwrap();
            self.header_count += 1;
        }

        self.storage_file.flush()?;
        Ok(())
    }

    pub fn start(mut self) {
        thread::spawn(move || {
            loop {
                let msg = self.incoming_control_receiver.recv().unwrap();

                match msg {
                    KalikoControlMessage::PeerAnnouncedHeight(height) => {
                        if (height as u32) <= self.num_headers() {
                            continue;
                        }

                        // Send message requesting new headers.
                        self.outgoing_control_sender.send(KalikoControlMessage::RequestHeaders(self.latest_header.hash())).unwrap();
                    },
                    KalikoControlMessage::NewHeadersAvailable(headers) => {
                        self.write_headers(&headers).unwrap();
                    },
                    _ => continue,
                }
            }
        });
    }
}