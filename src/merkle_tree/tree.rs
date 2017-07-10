use abc::*;


#[derive(Debug, Default)]
pub struct MerkleTree<D, T> where D: DataStorageReadonly, T: TreeStorage {
    data: D,
    tree: T,
}

impl <D, T> MerkleTree<D, T> where D: DataStorageReadonly, T: TreeStorage {
    /// Creates instance without checking of data integrity
    pub fn new_unchecked(data: D, tree: T) -> Self {
        MerkleTree { data, tree }
    }

    /// Creates instance and rebuilds the tree.
    /// The same as to call `new_unchecked` and then `rebuild`
    pub fn new_and_rebuild(data: D, tree: T) -> Self {
        let mut mt = MerkleTree { data, tree };
        mt.rebuild();
        mt
    }

    /// Returns a reference to the data storage
    pub fn data(&self) -> &D {
        &self.data
    }
    /// Returns a reference to the tree storage
    pub fn tree(&self) -> &T {
        &self.tree
    }

    /// For corruption tests only
    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut D {
        &mut self.data
    }
    /// For corruption tests only
    #[cfg(test)]
    pub fn tree_mut(&mut self) -> &mut T {
        &mut self.tree
    }

    /// Rebuilds full tree from scratch, using the current state of the data
    /// Will take a long time for a large dataset.
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

    /// Checks if the data corresponds to the checksum.
    /// Will take a long time for a large dataset.
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

    /// Checks data integrity of the tree.
    /// Will take a long time for a large tree.
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

    /// Checks the proof for a chain from a data block to the root
    pub fn audit_proof(&self, mut index: usize) -> Option<bool> {
        let data_hash = self.data.get(index)
            .map(|block| T::Algorithm::eval_hash(block));
        if data_hash.is_none() {
            return None;
        }
        let mut hash = self.tree.get_value(0, index).unwrap();
        let data_hash = data_hash.unwrap();
        if *hash != data_hash {
            return Some(false);
        }
        for level in 0 .. self.tree.len() - 1 {
            //let source = self.tree.get_value(level, index).unwrap();
            let index2 = index + (index + 1) % 2 - index % 2;
            let hash2 = self.tree.get_value(level, index2);
            let pair = match index < index2 {
                true => (hash, hash2.unwrap_or(hash)),
                false => (hash2.unwrap(), hash),
            };
            index /= 2;
            hash = self.tree.get_value(level + 1, index).unwrap();
            let pair_hash = T::Algorithm::eval_hash(&pair);
            if *hash != pair_hash {
                return Some(false);
            }
        }
        Some(true)
    }
}


impl <D, T> MerkleTree<D, T> where D: DataStorage, T: TreeStorage {
    /// Appends a new data block at the back of data chain
    pub fn push(&mut self, data: D::Block) {
        if !self.data.is_writeable() {
            unimplemented!()
        }
        let hash = T::Algorithm::eval_hash(&data);
        self.data.push(data).unwrap();
        self.push_hash(0, hash)
    }

    // Appends new hash to the level, creating a level if it did not exist
    fn push_hash(&mut self, level: usize, hash: <T::Algorithm as MTAlgorithm>::Value) {
        debug_assert!(level <= self.tree.len());
        if self.tree.len() == level {
            self.tree.grow();
        }
        // FIXME check error
        self.tree.push(level, hash).unwrap();
        self.update_branch(level)
    }

    // Updates last branch after new hash has been added to the level
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
}


#[cfg(test)]
mod tests {
    use abc::*;
    use super::MerkleTree;
    use fun::double::DoubleHash;
    use fun::sha256::Sha256;
    use data_storage::memory::MemoryReadonlyDataStorage;
    use data_storage::memory::MemoryDataStorage;
    use tree_storage::memory::MemoryTreeStorage;

    static DATA: [&[u8]; 3] = [b"123", b"321", b"555"];

    fn sample_ro_tree() -> MerkleTree<MemoryReadonlyDataStorage<'static, &'static [u8]>, MemoryTreeStorage<DoubleHash<Sha256>>> {
        MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(&DATA[..]), Default::default())
    }

    fn sample_rw_tree() -> MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<Sha256>>> {
        let mut tree = MerkleTree::default();
        tree.push(DATA[0]);
        tree.push(DATA[1]);
        tree.push(DATA[2]);
        tree
    }

    #[test]
    fn merkle_tree_works() {
        let tree = sample_ro_tree();
        // println!("{:?}", tree);
        // println!("{:?}", tree.root());
        let root = "SHA256:895073a7ee449758eec65efa6a9dcf51c41fa7b7a0e01b240c85d7fc230390e8";
        assert_eq!(format!("{:?}", tree.tree().get_root().unwrap()), root);
        assert!(tree.check_data().is_ok());
        assert!(tree.check_tree().is_ok());
        assert_eq!(tree.audit_proof(0), Some(true));
        assert_eq!(tree.audit_proof(1), Some(true));
        assert_eq!(tree.audit_proof(2), Some(true));
    }

    #[test]
    fn merkle_tree_check() {
        let mut tree = sample_rw_tree();
        assert!(tree.check_data().is_ok());
        assert!(tree.check_tree().is_ok());
        assert_eq!(tree.audit_proof(0), Some(true));
        assert_eq!(tree.audit_proof(1), Some(true));
        assert_eq!(tree.audit_proof(2), Some(true));

        // Damage data
        tree.data_mut().data_mut()[1] = b"666";
        assert!(tree.check_data().is_err());
        assert_eq!(tree.audit_proof(0), Some(true));
        assert_eq!(tree.audit_proof(1), Some(false));
        assert_eq!(tree.audit_proof(2), Some(true));

        let mut tree = sample_rw_tree();
        {
            // Damage tree
            let layers = tree.tree_mut().data_mut();
            layers[0][1] = layers[0][0];
        }
        assert!(tree.check_tree().is_err());
        assert_eq!(tree.audit_proof(0), Some(false));
        assert_eq!(tree.audit_proof(1), Some(false));
        assert_eq!(tree.audit_proof(2), Some(true));
    }

    #[test]
    fn merkle_tree_rebuilds() {
        // a - built step by step
        // b - built step by step and then rebuilt
        // c - built from ready data

        let a = sample_rw_tree();
        let mut b = sample_rw_tree();
        b.rebuild();
        let c = sample_ro_tree();

        // println!("\n TREE a:\n\n{:?}", a);
        // println!("\n TREE b:\n\n{:?}", b);
        // println!("\n TREE c:\n\n{:?}", c);

        assert_eq!(a.tree().get_root().unwrap(), b.tree().get_root().unwrap());
        assert_eq!(a.tree().get_root().unwrap(), c.tree().get_root().unwrap());
    }
}
