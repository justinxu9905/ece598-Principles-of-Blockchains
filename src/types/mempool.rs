/* Mempool */
use std::collections::HashMap;
use crate::{H256, Hashable};
use crate::types::transaction::SignedTransaction;

#[derive(Debug, Default, Clone)]
pub struct Mempool{
    pub tx_map: HashMap<H256, SignedTransaction>,
}

impl Mempool {
    pub fn new() -> Self {
        return Mempool{tx_map: HashMap::new()}
    }

    pub fn insert(&mut self, t: &SignedTransaction) {
        let t_hash = t.hash();
        self.tx_map.insert(t_hash, t.clone());
    }

    pub fn remove(&mut self, t: &SignedTransaction) {
        let t_hash = t.hash();
        if self.tx_map.contains_key(&t_hash) {
            self.tx_map.remove(&t_hash);
        }
    }
}