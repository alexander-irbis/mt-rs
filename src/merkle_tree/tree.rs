use prelude::*;


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
    pub fn new_and_rebuild(data: D, tree: T) -> Result<Self> {
        let mut mt = MerkleTree { data, tree };
        mt.rebuild()?;
        Ok(mt)
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
    pub fn rebuild(&mut self) -> Result<()> {
        if self.data.is_empty()? {
            return self.tree.clear_and_reserve(&[]);
        }

        let mut sizes = Vec::<usize>::new();
        let mut len = self.data.len()?;
        let mut layer_buffer = Vec::with_capacity(len);

        loop {
            sizes.push(len);
            if len == 1 {
                break;
            }
            len = len / 2 + len % 2;
        }
        self.tree.clear_and_reserve(&sizes)?;

        for block in self.data.iter()? {
            let hash = T::Algorithm::eval_hash(&block?);
            layer_buffer.push(hash);
        }
        self.tree.extend_from_slice(0, &layer_buffer)?;

        if sizes.len() < 2 {
            return Ok(());
        }

        for level in 0 .. sizes.len() - 1 {
            layer_buffer.clear();
            for chunk in self.tree.iter_level_by_pair(level)? {
                let hash = T::Algorithm::eval_hash(&chunk?);
                layer_buffer.push(hash);
            }
            self.tree.extend_from_slice(level + 1, &layer_buffer)?;
        }

        Ok(())
    }

    /// Checks if the data corresponds to the checksum.
    /// Will take a long time for a large dataset.
    pub fn check_data(&self) -> Result<()> {
        if self.data.is_empty()? && self.tree.is_empty()? {
            return Ok(());
        } else if self.data.is_empty()? || self.tree.is_empty()? {
            Err(StateError::InconsistentState)?;
        } else if self.data.len()? != self.tree.get_level_len(0)? {
            Err(StateError::InconsistentState)?;
        }
        for (block, cs) in self.data.iter()?.zip(self.tree.iter_level(0)?) {
            if T::Algorithm::eval_hash(&block?) != cs? {
                Err(StateError::DataDoesNotMatchTheChecksum)?;
            }
        }
        Ok(())
    }

    /// Checks data integrity of the tree.
    /// Will take a long time for a large tree.
    pub fn check_tree(&self) -> Result<()> {
        if self.tree.is_empty()? {
            return Ok(());
        } else if self.tree.len()? == 1 && self.tree.get_level_len(0)? == 1 {
            return Ok(());
        }
        for level in 0 .. self.tree.len()? - 1 {
            let source_len = self.tree.get_level_len(level)?;
            if self.tree.get_level_len(level + 1)? != source_len / 2 + source_len % 2 {
                Err(StateError::InconsistentState)?;
            }
        }
        for level in (0 .. self.tree.len()? - 1).rev() {
            let source = self.tree.iter_level_by_pair(level)?;
            let derived = self.tree.iter_level(level + 1)?;
            for (chunk, cs) in source.zip(derived) {
                if T::Algorithm::eval_hash(&chunk?) != cs? {
                    Err(StateError::DataDoesNotMatchTheChecksum)?;
                }
            }
        }
        Ok(())
    }

    /// Checks the proof for a chain from a data block to the root
    pub fn audit_proof(&self, mut index: usize) -> Result<bool> {
        let data_hash = T::Algorithm::eval_hash(&self.data.get(index)?);
        let mut hash = self.tree.get_value(0, index)?;
        if hash != data_hash {
            return Ok(false);
        }
        for level in 0 .. self.tree.len()? - 1 {
            //let source = self.tree.get_value(level, index).unwrap();
            let index2 = index + (index + 1) % 2 - index % 2;
            let hash2 = self.tree.get_value(level, index2).iob_is_ok()?;
            let pair = match index < index2 {
                false => (hash2.ok_or(INDEX_IS_OUT_OF_BOUNDS)?, hash),
                true => {
                    let hash2 = hash2.unwrap_or_else(|| hash.clone());
                    (hash, hash2)
                },
            };
            index /= 2;
            hash = self.tree.get_value(level + 1, index)?;
            let pair_hash = T::Algorithm::eval_hash(&pair);
            if hash != pair_hash {
                return Ok(false);
            }
        }
        Ok(true)
    }
}


impl <D, T> MerkleTree<D, T> where D: DataStorage, T: TreeStorage {
    fn check_if_data_is_writable(&self) -> Result<()> {
        if self.data.is_writeable() {
            Ok(())
        } else {
            Err(Error::new_ro("Data storage is not writable"))
        }
    }

    /// Appends a new data block at the back of data chain
    pub fn push(&mut self, data: D::Block) -> Result<()> {
        // TODO ensure that tree storage is writable
        self.check_if_data_is_writable()?;
        let hash = T::Algorithm::eval_hash(&data);
        self.data.push(data).unwrap();
        self.push_hash(0, hash)
    }

    // Appends new hash to the level, creating a level if it did not exist
    fn push_hash(&mut self, level: usize, hash: <T::Algorithm as MTAlgorithm>::Value) -> Result<()> {
        debug_assert!(level <= self.tree.len()?);
        if self.tree.len()? == level {
            self.tree.grow()?;
        }
        self.tree.push(level, hash)?;
        self.update_branch(level)
    }

    // Updates last branch after new hash has been added to the level
    fn update_branch(&mut self, level: usize) -> Result<()> {
        debug_assert!(level < self.tree.len()?);
        let layer_is_last = self.tree.len()? == level + 1;
        let len = self.tree.get_level_len(level)?;
        debug_assert!(len > 0);
        if len == 1 {
            return Ok(());
        }
        let hash = {
            let layer = &self.tree.get_level(level)?;
            let last = len - 1;
            let pair = ( layer.get(last - last % 2)?, layer.get(last)? );
            T::Algorithm::eval_hash(&pair)
        };
        let next_level = level + 1;
        if layer_is_last || len % 2 == 1 {
            self.push_hash(next_level, hash)
        } else {
            let next_len = self.tree.get_level_len(next_level)?;
            debug_assert!(next_len > 0);
            *self.tree.get_value_mut(next_level, next_len - 1)? = hash;
            self.update_branch(next_level)
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
        MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(&DATA[..]), Default::default()).unwrap()
    }

    fn sample_rw_tree() -> MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<Sha256>>> {
        let mut tree = MerkleTree::default();
        tree.push(DATA[0]).unwrap();
        tree.push(DATA[1]).unwrap();
        tree.push(DATA[2]).unwrap();
        tree
    }

    #[test]
    fn merkle_tree_works() {
        let tree = sample_ro_tree();
        // println!("{:?}", tree);
        // println!("{:?}", tree.root());
        let root = "SHA256:895073a7ee449758eec65efa6a9dcf51c41fa7b7a0e01b240c85d7fc230390e8";
        assert_eq!(format!("{:?}", tree.tree().get_root().unwrap().unwrap()), root);
        assert!(tree.check_data().is_ok());
        assert!(tree.check_tree().is_ok());
        assert_eq!(tree.audit_proof(0).unwrap(), true);
        assert_eq!(tree.audit_proof(1).unwrap(), true);
        assert_eq!(tree.audit_proof(2).unwrap(), true);
    }

    #[test]
    fn merkle_tree_check() {
        let mut tree = sample_rw_tree();
        assert!(tree.check_data().is_ok());
        assert!(tree.check_tree().is_ok());
        assert_eq!(tree.audit_proof(0).unwrap(), true);
        assert_eq!(tree.audit_proof(1).unwrap(), true);
        assert_eq!(tree.audit_proof(2).unwrap(), true);

        // Damage data
        tree.data_mut().data_mut()[1] = b"666";
        assert!(tree.check_data().is_err());
        assert_eq!(tree.audit_proof(0).unwrap(), true);
        assert_eq!(tree.audit_proof(1).unwrap(), false);
        assert_eq!(tree.audit_proof(2).unwrap(), true);

        let mut tree = sample_rw_tree();
        {
            // Damage tree
            let layers = tree.tree_mut().data_mut();
            layers[0][1] = layers[0][0];
        }
        assert!(tree.check_tree().is_err());
        assert_eq!(tree.audit_proof(0).unwrap(), false);
        assert_eq!(tree.audit_proof(1).unwrap(), false);
        assert_eq!(tree.audit_proof(2).unwrap(), true);
    }

    #[test]
    fn merkle_tree_rebuilds() {
        // a - built step by step
        // b - built step by step and then rebuilt
        // c - built from ready data

        let a = sample_rw_tree();
        let mut b = sample_rw_tree();
        b.rebuild().unwrap();
        let c = sample_ro_tree();

        // println!("\n TREE a:\n\n{:?}", a);
        // println!("\n TREE b:\n\n{:?}", b);
        // println!("\n TREE c:\n\n{:?}", c);

        assert_eq!(a.tree().get_root().unwrap(), b.tree().get_root().unwrap());
        assert_eq!(a.tree().get_root().unwrap(), c.tree().get_root().unwrap());
    }
}
