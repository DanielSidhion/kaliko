use ::KalikoControlMessage;
use network::headers::BlockHeader;
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;

pub struct BlockHeaderStorage {
    storage_file: File,
    chain: Vec<BlockHeader>,
    splits: Vec<Vec<BlockHeader>>,

    header_request_time: Option<Instant>,
    incoming_control_sender: Sender<KalikoControlMessage>,
    incoming_control_receiver: Receiver<KalikoControlMessage>,
    outgoing_control_sender: Sender<KalikoControlMessage>,
}

impl BlockHeaderStorage {
    pub fn new(storage_location: &str, outgoing_control_sender: Sender<KalikoControlMessage>) -> BlockHeaderStorage {
        let storage_file = OpenOptions::new().read(true).append(true).create(true).open(storage_location).unwrap();

        // TODO: read storage_file and build the blockchain again.
        let latest_header = BlockHeader::new_genesis();
        let chain = vec![latest_header];

        let (incoming_control_sender, incoming_control_receiver) = channel();

        BlockHeaderStorage {
            storage_file,
            chain,
            splits: vec![],

            header_request_time: None,
            incoming_control_sender,
            incoming_control_receiver,
            outgoing_control_sender,
        }
    }

    pub fn incoming_sender(&self) -> Sender<KalikoControlMessage> {
        self.incoming_control_sender.clone()
    }

    fn build_headers(&mut self, mut headers: Vec<BlockHeader>) {
        // The logic in build_headers assumes that all headers are part of the same chain, so we must ensure that before doing any work.
        let (chained_headers, _) = headers.iter().fold((true, Vec::<u8>::new()), |acc, header| {
            match acc.0 {
                false => (false, vec![]),
                true => {
                    match acc.1.len() {
                        0 => (true, header.hash()),
                        32 => {
                            if acc.1 != header.prev_block {
                                (false, vec![])
                            } else {
                                (true, header.hash())
                            }
                        },
                        // TODO: convert this to error?
                        _ => panic!("Should never have received different length here"),
                    }
                }
            }
        });

        if !chained_headers {
            info!("We received headers to build that are not all connected!");
            // TODO: return error?
            return;
        }

        // TODO: consider the case where we already have a split in the chain.
        if self.splits.len() != 0 {
            
        }

        // Find in our chain where is the block referenced by the current header's `prev_block`.
        let common_base_height = {
            let first_header = &headers[0];
            let prev_block_hash = first_header.prev_block;

            let prev_header = self.chain.iter().rev().enumerate().find(|(_, h)| h.hash() == prev_block_hash);
            if let None = prev_header {
                // If the block is never found, we just ignore the current headers.
                // TODO: instead of ignoring the current headers, send a getheaders command to the peer who sent us these headers - we may be on the wrong branch.
                return;
            }

            prev_header.unwrap().0
        };

        // If the first header builds upon the chain that we have, we can just accept those headers. However, if they are a split in the chain, we need to switch to that split if the received headers form a bigger chain. Otherwise, we need to track the split and only switch when we find the biggest split.
        if common_base_height == 0 {
            // Just add the current header to the chain.
            self.chain.append(&mut headers);
        } else {
            if headers.len() < common_base_height {
                // We still have the bigger chain.
                return;
            }

            let common_chain_size = self.chain.len() - common_base_height;

            if headers.len() > common_base_height {
                // Remove the smaller branch, and start adding the headers from the bigger chain.
                self.chain.truncate(common_chain_size);
                self.chain.append(&mut headers);
            } else {
                // Keep the split and start tracking it.
                let first_split = self.chain.split_off(common_chain_size);
                self.splits.push(first_split);
                self.splits.push(headers);
                return;
            }
        }
    }

    fn block_locator(&self) -> Vec<Vec<u8>> {
        // TODO: what to do when we have a split?
        let mut result = vec![];

        result.append(&mut self.chain.iter().rev().take(10).map(|b| b.hash()).collect::<Vec<Vec<u8>>>());

        let mut chain_iter = self.chain.iter().rev().skip(10);
        let mut step = 1;
        while let Some(header) = chain_iter.nth(step) {
            result.push(header.hash());
            step *= 2;
        }

        if result.iter().last().unwrap() != &self.chain[0].hash() {
            // Adding the genesis block to the locator object.
            result.push(self.chain[0].hash());
        }

        result
    }

    pub fn start(mut self) {
        thread::spawn(move || {
            loop {
                let msg = self.incoming_control_receiver.recv().unwrap();

                debug!("Got control message: {:?}", msg);
                match msg {
                    KalikoControlMessage::PeerAnnouncedHeight(peer, height) => {
                        // If we hold a bigger chain, we just don't care about checking that peer's headers.
                        if (height as usize) < self.chain.len() {
                            continue;
                        }

                        // Send message requesting headers so we can compare, and expand or detect splits in the chain.
                        self.outgoing_control_sender.send(KalikoControlMessage::RequestHeadersFromPeer(peer, self.block_locator())).unwrap();
                    },
                    KalikoControlMessage::NewHeadersAvailable(peer, headers) => {
                        debug!("Building headers...");
                        self.build_headers(headers);
                        debug!("New chain:");
                        for item in self.chain.iter() {
                            debug!("  {}", item);
                        }

                        // Send message requesting more headers just in case that peer has more headers for us.
                        self.outgoing_control_sender.send(KalikoControlMessage::RequestHeadersFromPeer(peer, self.block_locator())).unwrap();
                    },
                    _ => continue,
                }
            }
        });
    }
}