use abc::*;


#[derive(Debug, Default)]
pub struct MerkleTree<A, DS> where A: MTAlgorithm, DS: DataStorage {
    // Hashes are stored as layers
    // In the begin (index 0) is the bottom level 0 with hashes of the data
    // next layers keep hashes of previous levels, till the root
    layers: Vec<Vec<A::Value>>,
    data: DS,
}

impl <A, DS> MerkleTree<A, DS> where A: MTAlgorithm, DS: DataStorage {
    pub fn new(data: DS) -> Self {
        let mut tree = MerkleTree {
            layers: Vec::new(),
            data,
        };
        if !tree.data().is_empty() {
            tree.rebuild();
        }
        tree
    }

    pub fn data(&self) -> &DS {
        &self.data
    }

    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut DS {
        &mut self.data
    }

    pub fn push(&mut self, data: DS::Block) {
        let hash = A::eval_hash(&data);
        self.data.push(data).unwrap();
        self.push_hash(0, hash)
    }

    fn push_hash(&mut self, layer: usize, hash: A::Value) {
        debug_assert!(layer <= self.layers.len());
        if self.layers.len() == layer {
            self.layers.push(Vec::new());
        }
        self.layers[layer].push(hash);
        self.update_branch(layer)
    }

    fn update_branch(&mut self, layer: usize) {
        let layer_is_last = self.layers.len() == layer + 1;
        let len = self.layers[layer].len();
        if len > 1 {
            let hash = {
                let layer = &self.layers[layer];
                let pair = if len % 2 == 0 {
                    (&layer[len - 2], &layer[len - 1])
                } else {
                    (&layer[len - 1], &layer[len - 1])
                };
                A::eval_hash(&pair)
            };
            if layer_is_last || len % 2 == 1 {
                self.push_hash(layer + 1, hash);
            } else {
                {
                    let top_layer = &mut self.layers[layer + 1];
                    let top_len = top_layer.len();
                    debug_assert!(top_len > 0);
                    top_layer[top_len - 1] = hash;
                }
                self.update_branch(layer + 1)
            }
        }
    }

    pub fn try_root(&self) -> Option<&A::Value> {
        self.layers.iter().last().and_then(|layer| layer.iter().last())
    }

    pub fn root(&self) -> &A::Value {
        self.try_root().expect("Tree is empty")
    }

    pub fn rebuild(&mut self) {
        self.layers.clear();
        if self.data.is_empty() {
            return;
        }
        let mut layer = Vec::with_capacity(self.data.len());
        for block in self.data.iter() {
            let hash = A::eval_hash(block);
            layer.push(hash);
        }
        self.layers.push(layer);
        loop {
            let layer = {
                let source = self.layers.iter().last().unwrap();
                let len = source.len();
                debug_assert!(len > 0);
                if len == 1 {
                    break;
                }
                let mut layer = Vec::with_capacity(len / 2 + len % 2);
                for chunk in source.chunks(2) {
                    let hash = if chunk.len() == 2 {
                        A::eval_hash(&chunk)
                    } else {
                        A::eval_hash(&(&chunk[0], &chunk[0]))
                    };
                    layer.push(hash);
                }
                layer
            };
            self.layers.push(layer);
        }
    }

    pub fn check_data(&self) -> Result<(), CheckError> {
        if self.data.is_empty() && self.layers.is_empty() {
            return Ok(());
        } else if self.data.is_empty() || self.layers.is_empty() {
            return Err(CheckError::InconstistentState);
        } else if self.data.len() != self.layers[0].len() {
            return Err(CheckError::InconstistentState);
        }
        let layer = &self.layers[0];
        for (block, cs) in self.data.iter().zip(layer.iter()) {
            let hash = A::eval_hash(block);
            if hash != *cs {
                return Err(CheckError::DataDoesNotMatchTheChecksum);
            }
        }
        Ok(())
    }

    pub fn check_tree(&self) -> Result<(), CheckError> {
        if self.layers.is_empty() {
            return Ok(());
        } else if self.layers.len() == 1 && self.layers[0].len() == 1 {
            return Ok(());
        }
        for window in self.layers.windows(2).rev() {
            let (source, derived) = (&window[0], &window[1]);
            if derived.len() != source.len() / 2 + source.len() % 2 {
                return Err(CheckError::InconstistentState);
            }
        }
        for window in self.layers.windows(2).rev() {
            let (source, derived) = (&window[0], &window[1]);
            for (chunk, cs) in source.chunks(2).zip(derived.iter()) {
                let hash = if chunk.len() == 2 {
                    A::eval_hash(&chunk)
                } else {
                    A::eval_hash(&(&chunk[0], &chunk[0]))
                };
                if hash != *cs {
                    return Err(CheckError::DataDoesNotMatchTheChecksum);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MerkleTree;
    use fun::double::DoubleHash;
    use fun::sha256::Sha256;
    use data_storage::memory::MemoryDataStorage;

    static DATA: [&[u8]; 3] = [b"123", b"321", b"555"];

    fn sample_tree() -> MerkleTree<DoubleHash<Sha256>, MemoryDataStorage<&'static [u8]>> {
        let mut tree = MerkleTree::new(MemoryDataStorage::new());
        tree.push(DATA[0]);
        tree.push(DATA[1]);
        tree.push(DATA[2]);
        tree
    }

    #[test]
    fn merkle_tree_works() {
        let tree = sample_tree();
        // println!("{:?}", tree);
        // println!("{:?}", tree.root());
        let root = "SHA256:895073a7ee449758eec65efa6a9dcf51c41fa7b7a0e01b240c85d7fc230390e8";
        assert_eq!(format!("{:?}", tree.root()), root);
        assert!(tree.check_tree().is_ok());
        assert!(tree.check_data().is_ok());
    }

    #[test]
    fn merkle_tree_check() {
        let mut tree = sample_tree();
        tree.data_mut().data_mut()[1] = b"666";
        assert!(tree.check_data().is_err());

        let mut tree = sample_tree();
        tree.layers[0][0] = tree.layers[0][1];
        assert!(tree.check_tree().is_err());
    }

    #[test]
    fn merkle_tree_rebuilds() {
        let a = sample_tree();
        let mut b = sample_tree();
        b.rebuild();
        // println!("\n TREE a:\n\n{:?}", a);
        // println!("\n TREE b:\n\n{:?}", b);
        assert_eq!(a.root(), b.root());
    }
}
