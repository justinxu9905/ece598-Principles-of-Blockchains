use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::types::hash::{H256, Hashable};
use log::{debug, warn, error};
use std::thread;
use crate::{Blockchain, StatePerBlock};
use crate::types::block::{Block};
use crate::types::transaction::{self, Transaction, SignedTransaction};
use crate::types::mempool::Mempool;

#[cfg(any(test,test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test,test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    orphan_buffer: Arc<Mutex<HashMap<H256, Vec<Block>>>>,
    mempool: Arc<Mutex<Mempool>>,
    state_per_block: Arc<Mutex<StatePerBlock>>,
}


impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
        orphan_buffer: &Arc<Mutex<HashMap<H256, Vec<Block>>>>,
        mempool: &Arc<Mutex<Mempool>>,
        state_per_block: &Arc<Mutex<StatePerBlock>>,
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
            orphan_buffer: Arc::clone(orphan_buffer),
            mempool: Arc::clone(mempool),
            state_per_block: Arc::clone(state_per_block),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn tx_signature_check(&self, block: Block) -> bool {
        let txs = block.clone().content.data;
        let mut is_valid = true;
        for tx in txs {
            if !transaction::verify(&tx.transaction, &tx.public_key, &tx.signature) {
                is_valid = false;
                break;
            }
        }
        is_valid
    }

    fn worker_loop(&self) {
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }

                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }

                Message::NewBlockHashes(hashVec) => {
                    let mut msg = Vec::new();
                    let blockchain = self.blockchain.lock().unwrap();
                    for hs in hashVec {
                        if !blockchain.has(hs) {
                            msg.push(hs);
                        }
                    }
                    if !msg.is_empty() {
                        peer.write(Message::GetBlocks(msg));
                    }
                }

                Message::GetBlocks(hashVec) => {
                    let mut msg = Vec::new();
                    let blockchain = self.blockchain.lock().unwrap();
                    for hs in hashVec {
                        if blockchain.hash_block_map.contains_key(&hs) {
                            msg.push(blockchain.hash_block_map[&hs].clone());
                        }
                    }
                    if !msg.is_empty() {
                        peer.write(Message::Blocks(msg))
                    }
                }

                Message::Blocks(blockVec) => {
                    let mut blockchain = self.blockchain.lock().unwrap();
                    let mut orphan_buffer = self.orphan_buffer.lock().unwrap();
                    let mut mempool = self.mempool.lock().unwrap();
                    let mut new_blocks: Vec<H256> = vec![];
                    let mut missing_parents: Vec<H256> = vec![];
                    // let difficulty = blockchain.get(blockchain.tip()).get_difficulty();
                    let difficulty = blockchain.hash_block_map[&blockchain.tip()].get_difficulty();

                    for blk in blockVec {
                        let blk_hs = blk.hash();
                        // check if block.hash() <= difficulty
                        if blk_hs > difficulty {
                            continue;
                        }

                        // Parent check
                        let parent_hs = blk.get_parent();

                        // Check if the block's parent exists local copy of blockchain
                        if blockchain.has(parent_hs) {
                            if blk.get_difficulty() == difficulty {
                                if !blockchain.has(blk_hs) {
                                    if self.tx_signature_check(blk.clone()) {
                                        // update blockchain
                                        blockchain.insert(&blk);
                                        let mut state_per_block = self.state_per_block.lock().unwrap();
                                        state_per_block.update(&blk);
                                        drop(state_per_block);
                                        new_blocks.push(blk_hs);

                                        // update mempool
                                        for tx in &blk.content.data {
                                            mempool.tx_map.remove(&tx.hash());
                                        }
                                    }
                                }
                            }
                        } else { // If this check fails, also send GetBlocks message, containing this parent hash
                            missing_parents.push(parent_hs);
                            // Prepare for Orphan block handler
                            let mut children: Vec<Block> = vec![];
                            if orphan_buffer.contains_key(&parent_hs) {
                                children = orphan_buffer[&parent_hs].clone();
                            }
                            children.push(blk);
                            orphan_buffer.insert(parent_hs, children);
                        }
                    }
                    if !missing_parents.is_empty() {
                        peer.write(Message::GetBlocks(missing_parents));
                    }
                    // Orphan block handler
                    let mut left_new_blocks: Vec<H256> = vec![];
                    left_new_blocks = new_blocks.clone();
                    while !left_new_blocks.is_empty() {
                        let mut left_new_blocks1: Vec<H256> = vec![];
                        for blk_hs in left_new_blocks {
                            if orphan_buffer.contains_key(&blk_hs) {
                                let children = &orphan_buffer[&blk_hs];
                                for child in children {
                                    if child.get_difficulty() == difficulty {
                                        if self.tx_signature_check(child.clone()) {
                                            // update blockchain
                                            blockchain.insert(&child.clone());
                                            let mut state_per_block = self.state_per_block.lock().unwrap();
                                            state_per_block.update(&child);
                                            drop(state_per_block);
                                            left_new_blocks1.push(child.hash());
                                            new_blocks.push(child.hash());

                                            // update mempool
                                            for tx in &child.content.data {
                                                mempool.tx_map.remove(&tx.hash());
                                            }
                                        }
                                    }
                                }
                            }
                            orphan_buffer.remove(&blk_hs);
                        }
                        left_new_blocks = left_new_blocks1;
                    }

                    if !new_blocks.is_empty() {
                        self.server.broadcast(Message::NewBlockHashes(new_blocks));
                    }
                }

                Message::NewTransactionHashes(hashVec) => {
                    let mut new_tx_hashes = Vec::new();
                    let mut mempool = self.mempool.lock().unwrap();
                    for hs in hashVec.iter() {
                        if !mempool.tx_map.contains_key(hs) {
                            new_tx_hashes.push(hs.clone());
                        }
                    }
                    if new_tx_hashes.len() > 0 {
                        peer.write(Message::GetTransactions(new_tx_hashes));
                    }
                }

                Message::GetTransactions(hashVec) => {
                    let mut msg = Vec::new();
                    let mut mempool = self.mempool.lock().unwrap();
                    for hs in hashVec {
                        if mempool.tx_map.contains_key(&hs) {
                            msg.push(mempool.tx_map[&hs].clone());
                        }
                    }
                    if !msg.is_empty() {
                        peer.write(Message::Transactions(msg))
                    }
                }

                Message::Transactions(txVec) => {
                    let mut mempool = self.mempool.lock().unwrap();
                    let mut new_txs: Vec<H256> = vec![];
                    for tx in txVec {
                        if transaction::verify(&tx.transaction, &tx.public_key, &tx.signature) {
                            mempool.insert(&tx);
                        }
                    }
                }

                _ => {}
            }
        }
    }
}

// #[cfg(any(test,test_utilities))]
// struct TestMsgSender {
//     s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
// }
// #[cfg(any(test,test_utilities))]
// impl TestMsgSender {
//     fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
//         let (s,r) = smol::channel::unbounded();
//         (TestMsgSender {s}, r)
//     }

//     fn send(&self, msg: Message) -> PeerTestReceiver {
//         let bytes = bincode::serialize(&msg).unwrap();
//         let (handle, r) = peer::Handle::test_handle();
//         smol::block_on(self.s.send((bytes, handle))).unwrap();
//         r
//     }
// }
// #[cfg(any(test,test_utilities))]
// /// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
// fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
//     let blockchain = Blockchain::new();
//     let blockchain = Arc::new(Mutex::new(blockchain));
//     let (server, server_receiver) = ServerHandle::new_for_test();
//     let (test_msg_sender, msg_chan) = TestMsgSender::new();
//     let orphan_buffer: Arc<Mutex<HashMap<H256, Vec<Block>>>> = Arc::new(Mutex::new(HashMap::new()));
//     let worker = Worker::new(1, msg_chan, &server, &blockchain, &orphan_buffer);
//     worker.start();
//     let hash_vec = blockchain.lock().unwrap().all_blocks_in_longest_chain();
//     (test_msg_sender, server_receiver, hash_vec)
// }

// // DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

// #[cfg(test)]
// mod test {
//     use ntest::timeout;
//     use crate::types::block::generate_random_block;
//     use crate::types::hash::Hashable;

//     use super::super::message::Message;
//     use super::generate_test_worker_and_start;

//     #[test]
//     #[timeout(60000)]
//     fn reply_new_block_hashes() {
//         let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
//         let random_block = generate_random_block(v.last().unwrap());
//         let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
//         let reply = peer_receiver.recv();
//         if let Message::GetBlocks(v) = reply {
//             assert_eq!(v, vec![random_block.hash()]);
//         } else {
//             panic!();
//         }
//     }
//     #[test]
//     #[timeout(60000)]
//     fn reply_get_blocks() {
//         let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
//         let h = v.last().unwrap().clone();
//         let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
//         let reply = peer_receiver.recv();
//         if let Message::Blocks(v) = reply {
//             assert_eq!(1, v.len());
//             assert_eq!(h, v[0].hash())
//         } else {
//             panic!();
//         }
//     }
//     #[test]
//     #[timeout(60000)]
//     fn reply_blocks() {
//         let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
//         let random_block = generate_random_block(v.last().unwrap());
//         let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
//         let reply = server_receiver.recv().unwrap();
//         if let Message::NewBlockHashes(v) = reply {
//             assert_eq!(v, vec![random_block.hash()]);
//         } else {
//             panic!();
//         }
//     }
// }

// // DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
