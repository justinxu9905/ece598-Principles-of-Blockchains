pub mod worker;
use crate::blockchain::{self, *};
use crate::types::block::{Block, Content, Header};
use crate::types::merkle::MerkleTree;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::types::transaction::SignedTransaction;
use crate::types::mempool::Mempool;
use log::info;
use crate::types::hash::{Hashable, H256};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update,     // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>, //receiver of control signal
    operating_state: OperatingState,
    finished_block_chan: Sender<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

//double check
pub fn new(
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
) -> (Context, Handle, Receiver<Block>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_block_sender, finished_block_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool),
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

#[cfg(any(test, test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
    // See: https://piazza.com/class/kykjhx727ab1ge?cid=76
    // generate a new blockchain, wrap it in Arc and the Mutex, call new(&blockchain).
    // This test case expects the miner thread to be able to use its blockchain
    let blockchain = Blockchain::new();
    let mempool = Mempool::new();
    let blockchain = Arc::new(Mutex::new(blockchain));
    let mempool = Arc::new(Mutex::new(mempool));
    return new(&blockchain, &mempool);
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn miner_loop(&mut self) {
        // main mining loop
        let mut blockchain_ = self.blockchain.lock().unwrap();
        let mut parent_ = blockchain_.tip();
        //the difficulty of this block = difficulty of parent block.
        let difficulty_ = blockchain_.hash_block_map[&parent_].header.difficulty;
        drop(blockchain_);
        drop(difficulty_);
        drop(parent_);
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                //unimplemented!()
                                parent_ = self.blockchain.lock().unwrap().tip();
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }
            let mut signed_tx_ = Vec::<SignedTransaction>::new();
            let mut mempool = self.mempool.lock().unwrap();
            if mempool.tx_map.len() >= 10 {
                for tx_hs in mempool.tx_map.keys() {
                    signed_tx_.push(mempool.tx_map[&tx_hs].clone());
                }
                for tx in signed_tx_.clone() {
                    mempool.remove(&tx);
                }
                if !signed_tx_.is_empty() {
                    let mut rng = rand::thread_rng();
                    let timestamp_ = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();

                    let merkle_tree_ = MerkleTree::new(&signed_tx_.clone());
                    let merkle_root_ = merkle_tree_.root();
                    let nonce_: u32 = rng.gen();
                    let header_ = Header {
                        parent: parent_,
                        nonce: nonce_,
                        difficulty: difficulty_,
                        timestamp: timestamp_,
                        merkle_root: merkle_root_,
                    };
                    let content_ = Content { data: signed_tx_.clone() };
                    let mut block = Block {
                        header: header_,
                        content: content_,
                    };

                    if block.hash() <= difficulty_ {
                        self.finished_block_chan.send(block.clone()).expect("Send finished block error");
                        parent_ = block.hash();
                    }
                    
                    // loop {
                    //     block.header.nonce = nonce_;
                    //     // TODO for student: if block mining finished, you can have something like:
                    //     if block.hash() <= difficulty_ {
                    //         match self.finished_block_chan.send(block.clone()) {
                    //             Ok(()) => {
                    //                 parent_ = block.hash();
                    //             }
                    //             // See: https://doc.rust-lang.org/book/ch12-06-writing-to-stderr-instead-of-stdout.html
                    //             Err(e) => {
                    //                 eprintln!("Error: {}", e);
                    //             }
                    //         }
                    //         break;
                    //     }
                    // }
                }
            }
            // TODO for student: actual mining, create a block
            
            drop(mempool);

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use crate::types::hash::Hashable;
    use ntest::timeout;

    #[test]
    #[timeout(60000)]
    fn miner_three_block() { // may always fail cuz cannot mine empty block
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..2 {
            let block_next = finished_block_chan.recv().unwrap();
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
