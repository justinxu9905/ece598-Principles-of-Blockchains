use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::convert::TryFrom;
use std::rc::Rc;
use ring::digest::{Context};
use ring::digest;
use super::hash::{Hashable, H256};


#[derive(Debug, Default)]
pub struct MerkleNode {
    left:           Option<Rc<RefCell<MerkleNode>>>,
    right:          Option<Rc<RefCell<MerkleNode>>>,
    value:          H256,
}

impl MerkleNode {
    pub fn build_leaf(v: H256) -> MerkleNode {
        MerkleNode {
            left: None,
            right: None,
            value: v,
        }
    }

    pub fn build_node(l: &MerkleNode, r: &MerkleNode) -> MerkleNode {
        let mut ctx = digest::Context::new(&digest::SHA256);
        ctx.update(l.value.as_ref());
        ctx.update(r.value.as_ref());
        let cat_hash = ctx.finish();

        MerkleNode {
            left: Option::from(Rc::new(RefCell::new(MerkleNode{
                left: l.left.clone(),
                right: l.right.clone(),
                value: l.value
            }))),
            right: Option::from(Rc::new(RefCell::new(MerkleNode{
                left: r.left.clone(),
                right: r.right.clone(),
                value: r.value
            }))),
            value: H256::from(<[u8; 32]>::try_from(cat_hash.as_ref()).unwrap()),
        }
    }
}

impl Clone for MerkleNode {
    fn clone(&self) -> MerkleNode {
        MerkleNode {
            left: self.left.clone(),
            right: self.right.clone(),
            value: self.value.clone()
        }
    }
}

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    root:       MerkleNode,
    nodes:      BTreeMap<usize, VecDeque<MerkleNode>>,
    height:     usize,
    leaf_size:  usize,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        let mut nodes = BTreeMap::new();
        let mut height = 0;
        let leaf_size = data.len();

        if data.len() == 0 {
            return MerkleTree{
                root: MerkleNode{
                    left: None,
                    right: None,
                    value: Default::default(),
                },
                nodes,
                height,
                leaf_size
            }
        }

        let mut cur_lv: VecDeque<MerkleNode> = VecDeque::new();
        for v in data.iter() {
            // print!("{} ", v.hash());
            cur_lv.push_back(MerkleNode::build_leaf(v.hash()));
        }

        while cur_lv.len() > 1 {
            let mut up_lv: VecDeque<MerkleNode> = VecDeque::new();
            for i in (0..cur_lv.len()).step_by(2) {
                let l = cur_lv.get(i).unwrap();
                let r = cur_lv.get(i + 1).unwrap_or(l);
                let nd = MerkleNode::build_node(l, r);
                up_lv.push_back(nd);
            }
            nodes.insert(height, cur_lv);
            height += 1;
            cur_lv = up_lv;
        }
        nodes.insert(height, cur_lv.clone());

        MerkleTree{
            root: MerkleNode{
                left: cur_lv[0].left.clone(),
                right: cur_lv[0].right.clone(),
                value: cur_lv[0].value
            },
            nodes,
            height,
            leaf_size
        }
    }

    pub fn root(&self) -> H256 { self.root.value }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut prf: Vec<H256> = vec![];
        let mut cur_idx = index;
        let mut cur_h = 0;
        let mut cur_w = self.leaf_size;
        while cur_h < self.height {
            let cur_lv = self.nodes.get(&cur_h).unwrap();
            if cur_idx % 2 == 1 {
                prf.push(cur_lv.get(cur_idx-1).unwrap().value);
            } else if cur_idx != cur_w - 1 {
                prf.push(cur_lv.get(cur_idx+1).unwrap().value);
            }
            cur_idx = cur_idx / 2;
            cur_w = (cur_w + 1) / 2;
            cur_h += 1;
        }
        prf
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    let mut hs = *datum;
    let mut cur_idx = index;
    let mut cur_w = leaf_size;
    let mut p_vec = Vec::from(proof);
    p_vec.reverse();
    while cur_w > 1 {
        let mut ctx = digest::Context::new(&digest::SHA256);
        if cur_idx % 2 == 1 {
            let p = p_vec.pop().unwrap();
            ctx.update(p.as_ref());
            ctx.update(hs.as_ref());
        } else if cur_idx == cur_w - 1 {
            ctx.update(hs.as_ref());
            ctx.update(hs.as_ref());
        } else {
            let p = p_vec.pop().unwrap();
            ctx.update(hs.as_ref());
            ctx.update(p.as_ref());
        }
        hs = ctx.finish().into();
        cur_idx = cur_idx / 2;
        cur_w = (cur_w + 1) / 2;
    }
    hs == *root
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use crate::types::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }

}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
