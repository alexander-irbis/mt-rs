use abc::*;


#[derive(Debug, Default)]
pub struct MerkleTree<D, T> where D: DataStorage, T: TreeStorage {
    data: D,
    tree: T,
}

impl <D, T> MerkleTree<D, T> where D: DataStorage, T: TreeStorage {
    pub fn new_unchecked(data: D, tree: T) -> Self {
        MerkleTree { data, tree }
    }

    pub fn new_and_rebuild(data: D, tree: T) -> Self {
        let mut mt = MerkleTree { data, tree };
        mt.rebuild();
        mt
    }

    pub fn data(&self) -> &D {
        &self.data
    }
    pub fn tree(&self) -> &T {
        &self.tree
    }

    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut D {
        &mut self.data
    }
    #[cfg(test)]
    pub fn tree_mut(&mut self) -> &mut T {
        &mut self.tree
    }

    pub fn push(&mut self, data: D::Block) {
        let hash = T::Algorithm::eval_hash(&data);
        self.data.push(data).unwrap();
        self.push_hash(0, hash)
    }

    fn push_hash(&mut self, level: usize, hash: <T::Algorithm as MTAlgorithm>::Value) {
        debug_assert!(level <= self.tree.len());
        if self.tree.len() == level {
            self.tree.grow();
        }
        // FIXME check error
        self.tree.push(level, hash).unwrap();
        self.update_branch(level)
    }

    fn update_branch(&mut self, level: usize) {
        debug_assert!(level < self.tree.len());
        let layer_is_last = self.tree.len() == level + 1;
        let len = self.tree.get_level_len(level).unwrap();
        if len > 1 {
            let hash = {
                let layer = &self.tree.get_level(level).unwrap();
                let pair = if len % 2 == 0 {
                    (layer.get(len - 2).unwrap(), layer.get(len - 1).unwrap())
                } else {
                    (layer.get(len - 1).unwrap(), layer.get(len - 1).unwrap())
                };
                T::Algorithm::eval_hash(&pair)
            };
            let next_level = level + 1;
            if layer_is_last || len % 2 == 1 {
                self.push_hash(next_level, hash);
            } else {
                let next_len = self.tree.get_level_len(next_level).unwrap();
                debug_assert!(next_len > 0);
                *self.tree.get_value_mut(next_level, next_len - 1).unwrap() = hash;
                self.update_branch(next_level)
            }
        }
    }

    pub fn rebuild(&mut self) {
        if self.data.is_empty() {
            self.tree.clear_and_reserve(&[]);
            return;
        }

        let mut sizes = Vec::<usize>::new();
        let mut len = self.data.len();
        let mut layer_buffer = Vec::with_capacity(len);

        loop {
            sizes.push(len);
            if len == 1 {
                break;
            }
            len = len / 2 + len % 2;
        }
        self.tree.clear_and_reserve(&sizes);

        for block in self.data.iter() {
            let hash = T::Algorithm::eval_hash(block);
            layer_buffer.push(hash);
        }
        // FIXME check error
        self.tree.extend_from_slice(0, &layer_buffer).unwrap();

        if sizes.len() < 2 {
            return;
        }

        for level in 0 .. sizes.len() - 1 {
            layer_buffer.clear();
            for chunk in self.tree.iter_level_by_pair(level).unwrap() {
                let hash = T::Algorithm::eval_hash(&chunk);
                layer_buffer.push(hash);
            }
            // FIXME check error
            self.tree.extend_from_slice(level + 1, &layer_buffer).unwrap();
        }
    }

    pub fn check_data(&self) -> Result<(), CheckError> {
        if self.data.is_empty() && self.tree.is_empty() {
            return Ok(());
        } else if self.data.is_empty() || self.tree.is_empty() {
            return Err(CheckError::InconstistentState);
        } else if self.data.len() != self.tree.get_level_len(0).unwrap() {
            return Err(CheckError::InconstistentState);
        }
        if self.data.iter()
            .map(|block| T::Algorithm::eval_hash(block))
            .zip(self.tree.iter_level(0).unwrap())
            .any(|(hash, cs)| hash != *cs)
        {
            return Err(CheckError::DataDoesNotMatchTheChecksum);
        }
        Ok(())
    }

    pub fn check_tree(&self) -> Result<(), CheckError> {
        if self.tree.is_empty() {
            return Ok(());
        } else if self.tree.len() == 1 && self.tree.get_level_len(0).unwrap() == 1 {
            return Ok(());
        }
        for level in 0 .. self.tree.len() - 1 {
            let source_len = self.tree.get_level_len(level).unwrap();
            if self.tree.get_level_len(level + 1).unwrap() != source_len / 2 + source_len % 2 {
                return Err(CheckError::InconstistentState);
            }
        }
        for level in (0 .. self.tree.len() - 1).rev() {
            let source = self.tree.iter_level_by_pair(level).unwrap();
            let derived = self.tree.iter_level(level + 1).unwrap();
            if source
                .map(|chunk| T::Algorithm::eval_hash(&chunk))
                .zip(derived)
                .any(|(hash, cs)| hash != *cs)
            {
                return Err(CheckError::DataDoesNotMatchTheChecksum);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use abc::*;
    use super::MerkleTree;
    use fun::double::DoubleHash;
    use fun::sha256::Sha256;
    use data_storage::memory::MemoryDataStorage;
    use tree_storage::memory::MemoryTreeStorage;

    static DATA: [&[u8]; 3] = [b"123", b"321", b"555"];

    fn sample_tree() -> MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<Sha256>>> {
        let mut tree = MerkleTree::default();
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
        assert_eq!(format!("{:?}", tree.tree().root()), root);
        assert!(tree.check_tree().is_ok());
        assert!(tree.check_data().is_ok());
    }

    #[test]
    fn merkle_tree_check() {
        let mut tree = sample_tree();
        tree.data_mut().data_mut()[1] = b"666";
        assert!(tree.check_data().is_err());

        let mut tree = sample_tree();
        {
            let layers = tree.tree_mut().data_mut();
            layers[0][0] = layers[0][1];
        }
        assert!(tree.check_tree().is_err());
    }

    #[test]
    fn merkle_tree_rebuilds() {
        let a = sample_tree();
        let mut b = sample_tree();
        b.rebuild();
        // println!("\n TREE a:\n\n{:?}", a);
        // println!("\n TREE b:\n\n{:?}", b);
        assert_eq!(a.tree().root(), b.tree().root());
    }
}
