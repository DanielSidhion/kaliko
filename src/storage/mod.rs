use ::KalikoControlMessage;
use network::headers::BlockHeader;
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::Result;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;

struct BlockchainNode {
    header: BlockHeader,
    children: Vec<BlockchainNode>,
}

pub struct BlockHeaderStorage {
    storage_file: File,
    header_count: u32,
    chain: BlockchainNode,
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
        let header_count = 1;
        let chain = BlockchainNode {
            header: latest_header,
            children: vec![],
        };

        let (incoming_control_sender, incoming_control_receiver) = channel();

        BlockHeaderStorage {
            storage_file,
            header_count,
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
            // To keep using safe Rust, our search captures nodes and their parents in a tuple. This avoids parent references in BlockchainNode, which would likely cause a lot of complexity.
            // Actually, we might need to have a parent or figure out a way to simplify the code below (which is currently broken).
            let mut blocks_to_search: VecDeque<(&BlockchainNode, Option<&mut BlockchainNode>)> = VecDeque::new();
            blocks_to_search.push_back((&mut self.chain, None));
            while let Some(entry) = blocks_to_search.pop_front() {
                let (node, parent) = entry;

                if node.header.hash() == prev_block_hash {
                    // If the node we found already has a parent, check if we already have the header. If not, we found a split in the blockchain.
                    if let Some(mut block) = parent {
                        if let None = block.children.iter().find(|c| c.header.hash() == header.hash()) {
                            // Found a split in the blockchain. Add the current header as a child of `block`.
                            block.children.push(BlockchainNode {
                                header: *header,
                                children: vec![],
                            });
                        }
                    }
                        

                    break;
                }

                for child in &node.children {
                    blocks_to_search.push_back((child, Some(node)));
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
                        self.outgoing_control_sender.send(KalikoControlMessage::RequestHeaders(peer, self.chain.header.hash())).unwrap();
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