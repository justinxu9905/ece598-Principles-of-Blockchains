use crate::types::hash::{H256, Hashable};
use crate::types::block::{self, *};
use std::collections::{HashMap, HashSet};
use crate::types::merkle::MerkleTree;
use crate::types::transaction::SignedTransaction;
use hex_literal::hex;
use rand::Rng;
use rand::seq::IteratorRandom;
use crate::types::address::Address;


pub struct Blockchain {
    tip: H256,
    max_len: u128,
    pub hash_block_map: HashMap<H256, Block>,
    hash_len_map: HashMap<H256, u128>,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let parent_: H256 = [0u8; 32].into();
        let nonce_ = 0u32;
        let difficulty_: H256 =
            hex!("08812818230e0b3b608814e05e61fde06d0df794468a12162f287412df3ec890").into();
        let timestamp_ = 0u128;
        let mut tx_data: Vec<SignedTransaction> = Vec::new();
        let merkle_tree = MerkleTree::new(&tx_data);
        let tree_root = merkle_tree.root();
        let header_ = Header {
            parent: parent_,
            nonce: nonce_,
            difficulty: difficulty_,
            timestamp: timestamp_,
            merkle_root: tree_root,
        };
        let content_ = Content { data: tx_data };
        let genesis: Block = Block {
            header: header_,
            content: content_,
        };

        let mut tip: H256 = genesis.hash();
        let mut max_len: u128 = 1;
        let mut hash_block_map: HashMap<H256, Block> = HashMap::new();
        let mut hash_len_map: HashMap<H256, u128> = HashMap::new();
        hash_block_map.insert(tip, genesis.clone());
        hash_len_map.insert(tip, max_len);

        Blockchain {
            tip,
            max_len,
            hash_block_map,
            hash_len_map,
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let blk_hash = block.hash();
        self.hash_block_map.insert(blk_hash, block.clone());
        let parent = block.get_parent();
        let mut pre_len = 1;
        if self.hash_len_map.contains_key(&parent) {
            pre_len = self.hash_len_map[&parent];
        } 
        self.hash_len_map.insert(blk_hash, pre_len + 1);
        if pre_len + 1 > self.max_len {
            self.tip = blk_hash;
            self.max_len = pre_len + 1;
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.tip
    }

    pub fn has(&self, key: H256) -> bool {
        return self.hash_block_map.contains_key(&key);
    }

    pub fn all_transactions_in_longest_chain(&self) -> Vec<Vec<SignedTransaction>> {
        let mut p = self.tip;
        let mut tx_vec = Vec::new();
        for i in 0..self.max_len {
            tx_vec.push(self.hash_block_map[&p].get_tx());
            p = self.hash_block_map[&p].get_parent();
        }
        tx_vec.reverse();
        tx_vec
    }


    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        // unimplemented!()
        let mut chain: Vec<H256> = Vec::new();
        let mut p = self.tip();
        for i in 0..self.max_len{
            chain.push(p);
            p = self.hash_block_map[&p].get_parent();
        }
        chain.reverse();
        chain
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn m0() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());

    }



}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
