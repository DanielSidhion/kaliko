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

    header_request_time: Option<Instant>,
    incoming_control_sender: Sender<KalikoControlMessage>,
    incoming_control_receiver: Receiver<KalikoControlMessage>,
    outgoing_control_sender: Sender<KalikoControlMessage>,
}

impl BlockHeaderStorage {
    pub fn new(storage_location: &str, outgoing_control_sender: Sender<KalikoControlMessage>) -> BlockHeaderStorage {
        let mut storage_file = OpenOptions::new().read(true).append(true).create(true).open(storage_location).unwrap();

        // TODO: read storage_file and build the blockchain again.
        let latest_header = BlockHeader::new_genesis();
        let chain = vec![latest_header];

        let (incoming_control_sender, incoming_control_receiver) = channel();

        BlockHeaderStorage {
            storage_file,
            chain,
            
            header_request_time: None,
            incoming_control_sender,
            incoming_control_receiver,
            outgoing_control_sender,
        }
    }

    pub fn incoming_sender(&self) -> Sender<KalikoControlMessage> {
        self.incoming_control_sender.clone()
    }

    fn build_headers(&mut self, headers: &Vec<BlockHeader>) {
        for header in headers {
            let prev_block_hash = header.prev_block;

            // Find in our chain where is the block referenced by the current header's `prev_block`.
            // We do this by going through the chain using Breadth-First Search (BFS).
            let mut blocks_to_search = VecDeque::new();
            blocks_to_search.push_back(self.latest_header);
            while let Some(node_index) = blocks_to_search.pop_front() {
                if self.chain[node_index].data.hash() == prev_block_hash {
                    match self.chain[node_index].parent {
                        // If the node we found already has a parent, check if we already have the header. If not, we found a split in the blockchain. If we already have the header, we just do nothing.
                        Some(parent) => {
                            if let None = self.chain[parent].children.iter().find(|c| self.chain[**c].data.hash() == header.hash()) {
                                // Found a split in the blockchain. Add the current header as a child of `block`.
                                let new_header_index = self.chain.len();
                                self.chain.push(BlockchainNode {
                                    data: *header,
                                    parent: Some(parent),
                                    children: vec![],
                                });
                                self.chain[parent].children.push(new_header_index);
                            }
                        },
                        // If the node we found doesn't have a parent, that means this is a new header in the chain. Add it to the chain.
                        None => {
                            // TODO: turn the following 2 lines into a method.
                            let new_header_index = self.chain.len();
                            self.chain.push(BlockchainNode {
                                data: *header,
                                parent: None,
                                children: vec![],
                            });

                            self.chain[node_index].parent = Some(new_header_index);
                            self.latest_header = new_header_index;
                        }
                    }

                    // Since we found the node we were looking for, we can break out of the BFS loop.
                    break;
                }

                for child in &self.chain[node_index].children {
                    blocks_to_search.push_back(*child);
                }
            }
        }
    }

    pub fn start(mut self) {
        thread::spawn(move || {
            loop {
                let msg = self.incoming_control_receiver.recv().unwrap();

                match msg {
                    KalikoControlMessage::PeerAnnouncedHeight(peer, height) => {
                        if (height as u32) <= self.header_count {
                            continue;
                        }

                        // Send message requesting new headers.
                        // TODO: actually build the header chain that the protocol needs.
                        self.outgoing_control_sender.send(KalikoControlMessage::RequestHeaders(peer, self.chain[0].data.hash())).unwrap();
                    },
                    KalikoControlMessage::NewHeadersAvailable(headers) => {
                        self.build_headers(&headers);
                    },
                    _ => continue,
                }
            }
        });
    }
}