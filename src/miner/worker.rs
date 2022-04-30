use crate::blockchain::Blockchain;
use crate::network::message::Message;
use crate::network::server::Handle as ServerHandle;
use crate::types::block::Block;
use crate::types::hash::{Hashable, H256};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::StatePerBlock;

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
    state_per_block: Arc<Mutex<StatePerBlock>>
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: &Arc<Mutex<Blockchain>>,
        state_per_block: &Arc<Mutex<StatePerBlock>>
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain: Arc::clone(blockchain),
            state_per_block: Arc::clone(state_per_block),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _block = self
                .finished_block_chan
                .recv()
                .expect("Receive finished block error");
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            let mut blockchain_ = self.blockchain.lock().unwrap();
            blockchain_.insert(&_block);
            let mut state_per_block = self.state_per_block.lock().unwrap();
            state_per_block.update(&_block);
            drop(blockchain_);
            let mut v = Vec::new();
            v.push(_block.hash());
            self.server.broadcast(Message::NewBlockHashes(v));
        }
    }
}
