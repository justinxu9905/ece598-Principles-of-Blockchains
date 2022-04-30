pub(crate) mod worker;
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters, UnparsedPublicKey, ED25519};
use crate::blockchain::{self, *};
use crate::types::block::{Block, Content, Header};
use crate::types::merkle::MerkleTree;
use crate::types::address::Address;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::types::transaction::{SignedTransaction, Transaction, sign};
use crate::types::mempool::Mempool;
use log::info;
use crate::types::hash::{Hashable, H256};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use rand::{Rng, thread_rng};
use std::thread;
use std::time;
use ring::agreement::PublicKey;
use crate::types::state::StatePerBlock;
use crate::types::{key_pair, transaction};

enum ControlSignal {
    Start(u64), 
    Update,     
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>, 
    operating_state: OperatingState,
    tx_chan: Sender<SignedTransaction>,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
    state_per_block: Arc<Mutex<StatePerBlock>>,
    key_pair: Ed25519KeyPair,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

//double check
pub fn new(blockchain: &Arc<Mutex<Blockchain>>, mempool: &Arc<Mutex<Mempool>>, state_per_block: &Arc<Mutex<StatePerBlock>>) -> (Context, Handle, Receiver<SignedTransaction>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (tx_sender, tx_receiver) = unbounded();

    let mut rng = rand::thread_rng();
    let mut _source: [u8; 32] = rng.gen();
    let key_pair = Ed25519KeyPair::from_seed_unchecked(&_source).unwrap();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        tx_chan: tx_sender,
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool),
        state_per_block: Arc::clone(state_per_block),
        key_pair,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, tx_receiver)
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
            .name("tx".to_string())
            .spawn(move || {
                self.tx_loop();
            })
            .unwrap();
        info!("tx initialized into paused mode");
    }

    fn tx_loop(&mut self) {
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("tx shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("tx starting in continuous mode with lambda {}", i);
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
                                info!("tx shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("tx starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                //unimplemented!()
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

            let t: SignedTransaction = self.generate_valid_transaction();
            self.tx_chan.send(t.clone()).expect("Send random transaction error");

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }

    pub fn get_valid_source_and_amount_and_nonce(&self) -> (Address, u32, u32) {
        let blockchain = self.blockchain.lock().unwrap();
        let state_per_block = self.state_per_block.lock().unwrap();
        let tip = blockchain.tip();

        let latest_state = &state_per_block.hash_state_map[&blockchain.hash_block_map[&tip].hash()].clone();
        drop(blockchain);
        drop(state_per_block);

        let mut rng = rand::thread_rng();

        let mut valid_source: Address = Address::generate_random_address();
        let mut valid_amount: u32 = 0;
        let mut valid_nonce: u32 = 0;

        let mut loop_time: usize = 1;

        if latest_state.state.keys().len() == 0 {
            loop_time = 1
        } else {
            loop_time = rng.gen_range(0..latest_state.state.keys().len());
        }
        let mut counter = 0;

        for (account, nonce_and_balance) in latest_state.state.iter() {
            valid_source = *account;
            if (*nonce_and_balance).1 == 0 {
                valid_amount = 0;
            } else {
                valid_amount = rng.gen_range(0..(*nonce_and_balance).1);
            }
            valid_nonce = (*nonce_and_balance).0 + 1;
            counter += 1;
            if counter == loop_time {
                break;
            }
        }

        return (valid_source, valid_amount, valid_nonce);
    }

    fn get_valid_destination(&self) -> Address {
        let blockchain = self.blockchain.lock().unwrap();
        let tip = blockchain.tip();
        let state_per_block = self.state_per_block.lock().unwrap();
        let latest_state = &state_per_block.hash_state_map[&blockchain.hash_block_map[&tip].hash()].clone();
        drop(state_per_block);
        drop(blockchain);
        let mut rng = rand::thread_rng();
        let mut valid_destination: Address = Address::generate_random_address();
        let rand_val = rng.gen::<u8>();
        if rand_val % 2 == 0 && latest_state.state.keys().len() > 1{
            let mut loop_time = rng.gen_range(0..latest_state.state.keys().len());
            let mut counter = 0;
            for (account, nonce_and_balance) in latest_state.state.iter() {
                valid_destination = *account;
                counter += 1;
                if counter == loop_time {
                    break;
                }
            }
        }
        return valid_destination;
    }

    fn generate_valid_transaction(&self) -> SignedTransaction {
        let src_address_and_amount_and_nonce: (Address, u32, u32) = self.get_valid_source_and_amount_and_nonce();
        let sender = src_address_and_amount_and_nonce.0;
        let value = src_address_and_amount_and_nonce.1;
        let acc_nonce = src_address_and_amount_and_nonce.2;
        let key = key_pair::random();
        let public_key = key.public_key();

        let receiver: Address = self.get_valid_destination();

        let transaction = Transaction{sender, receiver, value, acc_nonce};
        let signature = sign(&transaction, &key);
        let t = SignedTransaction{
            transaction: transaction,
            signature: signature.as_ref().to_vec(),
            public_key: public_key.as_ref().to_vec()
        };
        return t;
    }
}




