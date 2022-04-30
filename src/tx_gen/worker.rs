use crossbeam::channel::{Receiver};
use log::{info};
use crate::network::server::Handle as ServerHandle;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::network::message::Message;
use crate::types::hash::{Hashable};
use crate::types::transaction::SignedTransaction;
use crate::types::mempool::Mempool;


#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    tx_chan: Receiver<SignedTransaction>,
    mempool: Arc<Mutex<Mempool>>
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        tx_chan: Receiver<SignedTransaction>,
        mempool: &Arc<Mutex<Mempool>>
    ) -> Self {
        Self {
            server: server.clone(),
            tx_chan,
            mempool: Arc::clone(mempool),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("mempool-worker".to_string())
            .spawn(move || {
                self.mempool_worker_loop();
            })
            .unwrap();
        info!("mempool-worker initialized into paused mode");
    }

    fn mempool_worker_loop(&self) {
        loop {
            let t: SignedTransaction = self.tx_chan.recv().expect("Receive finished block error");
            let mut mempool = self.mempool.lock().unwrap();
            mempool.insert(&t);
            drop(mempool);
            self.server.broadcast(Message::NewTransactionHashes(vec![t.hash()]));
        }
    }
}
