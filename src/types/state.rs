/* State */
use std::collections::HashMap;
use std::ops::Add;
use crate::{Block, H256, Hashable};
use crate::types::address::Address;
use crate::types::transaction::SignedTransaction;
use hex_literal::hex;

#[derive(Debug, Default, Clone)]
pub struct State {
    pub state: HashMap<Address, (u32, u32)>,
}

impl State {
    pub fn new() -> Self {
        return State{state: HashMap::new()}
    }

    pub fn check(&self, t: &SignedTransaction) -> bool {
        let tx = t.transaction.clone();
        let sender = tx.sender;
        let nonce = tx.acc_nonce;
        let value = tx.value;

        if !self.state.contains_key(&sender) {
            return false;
        }

        let new_sender_nonce = self.state[&sender].0 + 1;

        if nonce != new_sender_nonce || self.state[&sender].1 < value {
            println!("can't update invalid tx");
            return false;
        }

        return true
    }

    pub fn update(&mut self, t: &SignedTransaction) -> bool {
        let tx = t.transaction.clone();
        let sender = tx.sender;
        let nonce = tx.acc_nonce;
        let receiver = tx.receiver;
        let value = tx.value;

        if !self.state.contains_key(&sender) {
            return false;
        }

        let new_sender_nonce = self.state[&sender].0 + 1;

        if nonce != new_sender_nonce || self.state[&sender].1 < value {
            //println!("can't update invalid tx");
            return false;
        }

        let new_sender_balance = self.state[&sender].1 - value;

        self.state.insert(sender, (new_sender_nonce, new_sender_balance));

        let mut new_receiver_nonce = 0;
        let mut new_receiver_balance = value;
        if self.state.contains_key(&receiver) {
            new_receiver_nonce = self.state[&receiver].0;
            new_receiver_balance = new_receiver_balance + self.state[&receiver].1;
        }

        self.state.insert(receiver, (new_receiver_nonce, new_receiver_balance));
        return true;
    }

    pub fn to_vec_string(&self) -> Vec<String> {
        let mut res: Vec<String> = vec![];

        for (address, nonce_and_balance) in self.state.iter() {
            let cur_str = address.to_string() + ", " + &nonce_and_balance.0.to_string() + ", " + &nonce_and_balance.1.to_string();
            res.push(cur_str);
        }
        res.sort();
        return res;
    }
}



#[derive(Debug, Default, Clone)]
pub struct StatePerBlock {
    pub hash_state_map: HashMap<H256, State>,
}

impl StatePerBlock {
    pub fn new(tip: H256) -> Self {
        let mut init_state = State::new();
        let init_acc: Address = hex!("1234567812345678123456781234567812345678").into();
        init_state.state.insert(init_acc, (0, 1000));
        let mut map: HashMap<H256, State> = HashMap::new();
        map.insert(tip, init_state);
        StatePerBlock{
            hash_state_map: map,
        }
    }

    pub fn update(&mut self, block: &Block) {
        let mut new_state = self.hash_state_map[&block.get_parent()].clone();
        let txs = block.clone().content.data;

        for tx in &txs {
            new_state.update(&tx);
        }

        self.hash_state_map.insert(block.hash(), new_state);
    }
}
