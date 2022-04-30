use serde::{Serialize, Deserialize};
use crate::types::hash::{H256, Hashable};
use ring::digest::{digest, SHA256};
use rand::Rng;
use crate::types::transaction::SignedTransaction;
use crate::types::merkle::MerkleTree;
use std::time::{SystemTime, UNIX_EPOCH};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub content: Content,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkle_root: H256
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub data: Vec<SignedTransaction>,
}


impl Hashable for Header {
    fn hash(&self) -> H256 {
        digest(&SHA256, &bincode::serialize(&self).unwrap()).into()
    }
}


impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Block {
    pub fn get_parent(&self) -> H256 {
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        self.header.difficulty
    }

    pub fn get_tx(&self) -> Vec<SignedTransaction> { 
        return self.content.data.clone(); 
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    let mut rng = rand::thread_rng();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut t_vec: Vec<SignedTransaction> = Vec::new();
    let mut merkle_tree = MerkleTree::new(&t_vec);
    Block {
        header: Header {
            parent: *parent,
            nonce: rng.gen(),
            difficulty: <H256>::from([7u8; 32]),
            timestamp: now,
            merkle_root: merkle_tree.root()
        },
        content: Content {
            data: t_vec
        }
    }
}
