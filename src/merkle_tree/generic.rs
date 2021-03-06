use prelude::*;


#[derive(Debug, Default)]
pub struct MerkleTree<D, T> where D: DataStorageReadonly, T: TreeStorage {
    data: D,
    tree: T,
}

impl <D, T> MerkleTree<D, T> where D: DataStorageReadonly, T: TreeStorage {
    /// Creates an instance without checking of data integrity
    pub fn new_unchecked(data: D, tree: T) -> Self {
        MerkleTree { data, tree }
    }

    /// Creates an instance and checks both the data and the tree.
    /// The same as to call `new_unchecked` and then `check_tree` and `check_data`
    pub fn new_and_check(data: D, tree: T) -> Result<Self> {
        let mt = MerkleTree::new_unchecked(data, tree);
        mt.check_tree()?;
        mt.check_data()?;
        Ok(mt)
    }

    /// Creates an instance and rebuilds the tree.
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
    /// Returns found chain
    pub fn audit_proof(&self, mut index: usize) -> Result<Vec<<T::Algorithm as MTAlgorithm>::Value>> {
        let data_hash = T::Algorithm::eval_hash(&self.data.get(index)?);
        let mut hash = self.tree.get_value(0, index)?;
        if hash != data_hash {
            Err(StateError::DataDoesNotMatchTheChecksum)?;
        }
        let mut path = Vec::with_capacity(self.tree.len()?);
        path.push(hash.clone());
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
            if hash != T::Algorithm::eval_hash(&pair) {
                Err(StateError::DataDoesNotMatchTheChecksum)?;
            }
            path.push(hash.clone());
        }
        Ok(path)
    }

    /// returns just path; may be zipped with `.audit_path()`
    // TODO eventually replace Box with `-> impl Iterator<....>`
    pub fn audit_path_indexes(&self, mut index: usize) -> Result<Box<Iterator<Item=(usize, usize)>>> {
        if self.data.len()? <= index {
            Err(AccessError::IndexIsOutOfBounds)?;
        }
        Ok(Box::new((0 .. self.tree.len()?)
            .map(move |level| {
                index = index / 2;
                (level, index)
            })))
    }

    /// Returns the root hash, or None if tree is empty.
    pub fn get_root(&self) -> Result<Option<<T::Algorithm as MTAlgorithm>::Value>> {
        self.tree.get_root()
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

    /// Clears all data
    pub fn clear(&mut self) -> Result<()> {
        self.data.clear()?;
        self.tree.clear()
    }

    /// Appends a new data block at the back of data chain
    pub fn push(&mut self, data: D::DataValue) -> Result<()> {
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
        self.update_branch(level, true)
    }

    // Updates last branch after new hash has been added to the level
    fn update_branch(&mut self, level: usize, pushed: bool) -> Result<()> {
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
        if layer_is_last || pushed && len % 2 == 1 {
            self.push_hash(next_level, hash)
        } else {
            let next_len = self.tree.get_level_len(next_level)?;
            debug_assert!(next_len > 0);
            *self.tree.get_value_mut(next_level, next_len - 1)? = hash;
            self.update_branch(next_level, false)
        }
    }

    /// Appends a new data block at the back of data chain
    pub fn extend<DD: IntoIterator<Item=Result<D::DataValue>>>(&mut self, data: DD) -> Result<()> {
        let len = self.data.len()?;
        self.data.extend(data.into_iter())?;
        let new_len = self.data.len()?;
        if new_len - len == 0 {
            return Ok(());
        }
        let hashes: Vec<_> = self.data.range(len..new_len)?
            .map(|data| Ok(<T::Algorithm as MTAlgorithm>::eval_hash(&data?)))
            .collect();
        self.push_hashes_bulk(0, hashes)
    }

    // Appends new hashes to the level, creating a level if it did not exist
    fn push_hashes_bulk<VV: IntoIterator<Item=Result<<T::Algorithm as MTAlgorithm>::Value>>>(&mut self, level: usize, hashes: VV) -> Result<()> {
        debug_assert!(level <= self.tree.len()?);
        if self.tree.len()? == level {
            self.tree.grow()?;
        }
        let len = self.tree.get_level_len(level)?;
        self.tree.extend(level, hashes.into_iter())?;
        self.update_branch_bulk(level, len, true)
    }

    // Updates changed branches after new hashes has been added to the level
    fn update_branch_bulk(&mut self, level: usize, from: usize, pushed: bool) -> Result<()> {
        debug_assert!(level < self.tree.len()?);
        let len = self.tree.get_level_len(level)?;
        debug_assert!(len > 0);
        debug_assert!(from < len);
        if len - from == 1 {
            return self.update_branch(level, pushed)
        }
        let mut hashes = self.tree.iter_level_by_pair(level)?.skip(from / 2)
            .map(|chunk| Ok(<T::Algorithm as MTAlgorithm>::eval_hash(&chunk?)))
            .collect::<Vec<_>>()
            .into_iter();

        let layer_is_last = self.tree.len()? == level + 1;
        let next_level = level + 1;

        if layer_is_last || pushed && from % 2 == 0 {
            self.push_hashes_bulk(next_level, hashes)
        } else {
            let next_len = self.tree.get_level_len(next_level)?;
            debug_assert!(next_len > 0);
            *self.tree.get_value_mut(next_level, next_len - 1)? = hashes.next().unwrap()?;
            self.tree.extend(next_level, hashes)?;
            self.update_branch_bulk(next_level, from / 2, false)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::MerkleTree;
    use fun::double::DoubleHash;
    use fun::sha256::Sha256;
    use fun::sha256::Sha256Value;
    use data_storage::memory::MemoryReadonlyDataStorage;
    use data_storage::memory::MemoryDataStorage;
    use tree_storage::memory::MemoryTreeStorage;
    use util::hex2buf;

    static DATA: [&[u8]; 3] = [b"123", b"321", b"555"];

    const H00: &str = "5a77d1e9612d350b3734f6282259b7ff0a3f87d62cfef5f35e91a5604c0490a3";
    const H01: &str = "a13425677ad1e0a86ecef2f35a8eb276feb0e78e939ea5060c9de6c68592189c";
    const H02: &str = "164904585a9c3493aa5ae0c0ab0dc971c0e7c92ff0bac4d0f50c48496728f440";

    const H10: &str = "d6684cae6572ffb6c56dbeba44aa9e0d24e38d8d4daadbc278e883e3b45f9af3";
    const H11: &str = "2289c8389e55a9ffbec27d9075e4752ccfe82a702738b1718335fcb276c8445a";

    const H20: &str = "895073a7ee449758eec65efa6a9dcf51c41fa7b7a0e01b240c85d7fc230390e8";


    fn sha256(s: &str) -> Sha256Value {
        assert!(s.len() == 64);
        let mut buf = [0u8; 32];
        hex2buf(&mut buf[..], s).unwrap();
        Sha256Value(buf)
    }

    fn cmp_proof(a: &[&str], b: &[Sha256Value]) -> bool {
        a.len() == b.len() &&
            a.iter().cloned().map(sha256)
                .zip(b.iter())
                .all(|(x, &y)| x == y)
    }

    fn sample_ro_tree() -> MerkleTree<MemoryReadonlyDataStorage<'static, &'static [u8]>, MemoryTreeStorage<DoubleHash<Sha256>>> {
        MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::with_data(&DATA[..]), Default::default()).unwrap()
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
        let a = sample_ro_tree();
        // println!("{:?}", tree);
        // println!("{:?}", tree.root());
        assert_eq!(a.get_root().unwrap().unwrap(), sha256(H20));
        assert!(a.check_data().is_ok());
        assert!(a.check_tree().is_ok());
        assert!(cmp_proof(&[H00, H10, H20], &a.audit_proof(0).unwrap()));
        assert!(cmp_proof(&[H01, H10, H20], &a.audit_proof(1).unwrap()));
        assert!(cmp_proof(&[H02, H11, H20], &a.audit_proof(2).unwrap()));

        let mut b = sample_rw_tree();
        b.clear().unwrap();
        b.extend([DATA[0], DATA[1], DATA[2]].into_iter().cloned().map(Ok)).unwrap();
        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());

        b.clear().unwrap();
        b.extend([DATA[0], DATA[1]].into_iter().cloned().map(Ok)).unwrap();
        b.extend([DATA[2]].into_iter().cloned().map(Ok)).unwrap();
        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());

        b.clear().unwrap();
        b.extend([DATA[0]].into_iter().cloned().map(Ok)).unwrap();
        b.extend([DATA[1], DATA[2]].into_iter().cloned().map(Ok)).unwrap();
        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());

        b.clear().unwrap();
        b.extend([DATA[0]].into_iter().cloned().map(Ok)).unwrap();
        b.extend([DATA[1]].into_iter().cloned().map(Ok)).unwrap();
        b.extend([DATA[2]].into_iter().cloned().map(Ok)).unwrap();
        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());

        b.clear().unwrap();
        assert!(b.get_root().unwrap().is_none());
    }

    #[test]
    fn merkle_tree_check() {
        let mut tree = sample_rw_tree();
        assert!(tree.check_data().is_ok());
        assert!(tree.check_tree().is_ok());
        assert!(cmp_proof(&[H00, H10, H20], &tree.audit_proof(0).unwrap()));
        assert!(cmp_proof(&[H01, H10, H20], &tree.audit_proof(1).unwrap()));
        assert!(cmp_proof(&[H02, H11, H20], &tree.audit_proof(2).unwrap()));

        // Damage data
        tree.data_mut().data_mut()[1] = b"666";
        assert!(tree.check_data().is_err());
        assert!(cmp_proof(&[H00, H10, H20], &tree.audit_proof(0).unwrap()));
        assert!(tree.audit_proof(1).is_err());
        assert!(cmp_proof(&[H02, H11, H20], &tree.audit_proof(2).unwrap()));

        let mut tree = sample_rw_tree();
        {
            // Damage tree
            let layers = tree.tree_mut().data_mut();
            layers[0][1] = layers[0][0];
        }
        assert!(tree.check_tree().is_err());
        assert!(tree.audit_proof(0).is_err());
        assert!(tree.audit_proof(1).is_err());
        assert!(cmp_proof(&[H02, H11, H20], &tree.audit_proof(2).unwrap()));
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

        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());
        assert_eq!(a.get_root().unwrap(), c.get_root().unwrap());
    }
}
