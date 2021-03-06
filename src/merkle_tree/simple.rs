use prelude::*;


#[derive(Debug)]
pub struct MerkleTreeSimple<D, A> where A: MTAlgorithm, D: MTHash {
    data: Vec<D>,
    tree: Vec<Vec<A::Value>>,
}

impl <A, D> Default for MerkleTreeSimple<D, A> where A: MTAlgorithm, D: MTHash {
    fn default() -> Self {
        MerkleTreeSimple::new()
    }
}

impl <A, D> MerkleTreeSimple<D, A> where A: MTAlgorithm, D: MTHash {
    /// Creates an empty instance
    pub fn new() -> Self {
        MerkleTreeSimple { data: Vec::new(), tree: Default::default() }
    }

    /// Creates an instance without checking of data integrity
    pub fn new_unchecked<DD: Into<Vec<D>>, TT: Into<Vec<Vec<A::Value>>>>(data: DD, tree: TT) -> Self {
        MerkleTreeSimple { data: data.into(), tree: tree.into() }
    }

    /// Creates an instance and checks both the data and the tree.
    /// The same as to call `new_unchecked` and then `check_tree` and `check_data`
    pub fn new_and_check<DD: Into<Vec<D>>, TT: Into<Vec<Vec<A::Value>>>>(data: DD, tree: TT) -> Result<Self> {
        let mt = MerkleTreeSimple::new_unchecked(data, tree);
        mt.check_tree()?;
        mt.check_data()?;
        Ok(mt)
    }

    /// Creates an instance and rebuilds the tree.
    /// The same as to call `new_unchecked` and then `rebuild`
    pub fn new_and_rebuild<DD: Into<Vec<D>>>(data: DD) -> Self {
        let mut mt = MerkleTreeSimple::new_unchecked(data, Vec::new());
        mt.rebuild();
        mt
    }

    /// Returns a reference to the data
    pub fn data(&self) -> &[D] {
        &self.data
    }
    /// Returns a reference to the tree levels
    pub fn tree(&self) -> &[Vec<A::Value>] {
        &self.tree
    }

    /// Rebuilds full tree from scratch, using the current state of the data
    /// Will take a long time for a large dataset.
    pub fn rebuild(&mut self) {
        if self.data.is_empty() {
            self.tree.clear();
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

        for block in self.data.iter() {
            let hash = A::eval_hash(block);
            layer_buffer.push(hash);
        }
        self.tree.push(layer_buffer);

        if sizes.len() < 2 {
            return;
        }

        for level in 0 .. sizes.len() - 1 {
            let mut layer_buffer = Vec::with_capacity(sizes[level]);
            for chunk in self.tree[level].chunks(2) {
                let i2 = (chunk.len() + 1) % 2;
                let chunk = (&chunk[0], &chunk[i2]);
                let hash = A::eval_hash(&chunk);
                layer_buffer.push(hash);
            }
            self.tree.push(layer_buffer);
        }
    }

    /// Checks if the data corresponds to the checksum.
    /// Will take a long time for a large dataset.
    pub fn check_data(&self) -> Result<()> {
        if self.data.is_empty() && self.tree.is_empty() {
            return Ok(());
        } else if self.data.is_empty() || self.tree.is_empty() {
            Err(StateError::InconsistentState)?;
        } else if self.data.len() != self.tree[0].len() {
            Err(StateError::InconsistentState)?;
        }
        for (block, cs) in self.data.iter().zip(self.tree[0].iter()) {
            if A::eval_hash(&block) != *cs {
                Err(StateError::DataDoesNotMatchTheChecksum)?;
            }
        }
        Ok(())
    }

    /// Checks data integrity of the tree.
    /// Will take a long time for a large tree.
    pub fn check_tree(&self) -> Result<()> {
        if self.tree.is_empty() {
            return Ok(());
        } else if self.tree.len() == 1 && self.tree[0].len() == 1 {
            return Ok(());
        }
        for level in 0 .. self.tree.len() - 1 {
            let source_len = self.tree[level].len();
            if self.tree[level + 1].len() != source_len / 2 + source_len % 2 {
                Err(StateError::InconsistentState)?;
            }
        }
        for level in (0 .. self.tree.len() - 1).rev() {
            let source = self.tree[level].chunks(2);
            let derived = self.tree[level + 1].iter();
            for (chunk, cs) in source.zip(derived) {
                let i2 = (chunk.len() + 1) % 2;
                let chunk = (&chunk[0], &chunk[i2]);
                if A::eval_hash(&chunk) != *cs {
                    Err(StateError::DataDoesNotMatchTheChecksum)?;
                }
            }
        }
        Ok(())
    }

    /// Checks the proof for a chain from a data block to the root
    /// Returns found chain
    pub fn audit_proof(&self, mut index: usize) -> Result<Vec<&A::Value>> {
        let data = self.data.get(index).ok_or(AccessError::IndexIsOutOfBounds)?;
        let data_hash = A::eval_hash(&data);
        let mut hash = self.tree.get(0).and_then(|layer| layer.get(index))
            .ok_or(StateError::InconsistentState)?;
        if *hash != data_hash {
            Err(StateError::DataDoesNotMatchTheChecksum)?;
        }
        let mut path = Vec::with_capacity(self.tree.len());
        path.push(hash);
        for level in 0 .. self.tree.len() - 1 {
            let index2 = index + (index + 1) % 2 - index % 2;
            let hash2 = self.tree.get(level).and_then(|layer| layer.get(index2));
            let pair = match index < index2 {
                false => (hash2.ok_or(StateError::InconsistentState)?, hash),
                true => (hash, hash2.unwrap_or(hash)),
            };
            index /= 2;
            hash = self.tree.get(level + 1).and_then(|layer| layer.get(index))
                .ok_or(StateError::InconsistentState)?;
            if *hash != A::eval_hash(&pair) {
                Err(StateError::DataDoesNotMatchTheChecksum)?;
            }
            path.push(hash);
        }
        Ok(path)
    }

    /// returns just path; may be zipped with `.audit_path()`
    // TODO eventually replace Box with `-> impl Iterator<....>`
    pub fn audit_path_indexes(&self, mut index: usize) -> Result<Box<Iterator<Item=(usize, usize)>>> {
        if self.data.len() <= index {
            Err(AccessError::IndexIsOutOfBounds)?;
        }
        Ok(Box::new((0 .. self.tree.len())
            .map(move |level| {
                index = index / 2;
                (level, index)
            })))
    }

    /// Clears all data
    pub fn clear(&mut self) {
        self.data.clear();
        self.tree.clear();
    }

    /// Appends a new data block at the back of data chain
    pub fn push(&mut self, data: D) {
        let hash = A::eval_hash(&data);
        self.data.push(data);
        self.push_hash(0, hash)
    }

    // Appends new hash to the level, creating a level if it did not exist
    fn push_hash(&mut self, level: usize, hash: A::Value) {
        debug_assert!(level <= self.tree.len());
        if self.tree.len() == level {
            self.tree.push(Default::default());
        }
        self.tree[level].push(hash);
        self.update_branch(level, true)
    }

    // Updates last branch after new hash has been added to the level
    fn update_branch(&mut self, level: usize, pushed: bool) {
        debug_assert!(level < self.tree.len());
        let layer_is_last = self.tree.len() == level + 1;
        let len = self.tree[level].len();
        debug_assert!(len > 0);
        if len == 1 {
            return;
        }
        let hash = {
            let layer = &self.tree[level];
            let last = len - 1;
            let pair = ( &layer[last - last % 2], &layer[last] );
            A::eval_hash(&pair)
        };
        let next_level = level + 1;
        if layer_is_last || pushed && len % 2 == 1 {
            return self.push_hash(next_level, hash);
        }
        let next_len = self.tree[next_level].len();
        debug_assert!(next_len > 0);
        self.tree[next_level][next_len - 1] = hash;
        self.update_branch(next_level, false)
    }

    /// Appends a new data block at the back of data chain
    pub fn extend<DD: IntoIterator<Item=D>>(&mut self, data: DD) {
        let len = self.data.len();
        self.data.extend(data.into_iter());
        if self.data.len() - len == 0 {
            return;
        }
        let hashes: Vec<_> = self.data[len..].iter().map(|data| A::eval_hash(&data)).collect();
        self.push_hashes_bulk(0, hashes)
    }

    // Appends new hashes to the level, creating a level if it did not exist
    fn push_hashes_bulk<VV: IntoIterator<Item=A::Value>>(&mut self, level: usize, hashes: VV) {
        debug_assert!(level <= self.tree.len());
        if self.tree.len() == level {
            self.tree.push(Default::default());
        }
        let len = self.tree[level].len();
        self.tree[level].extend(hashes.into_iter());
        self.update_branch_bulk(level, len, true)
    }

    // Updates changed branches after new hashes has been added to the level
    fn update_branch_bulk(&mut self, level: usize, from: usize, pushed: bool) {
        debug_assert!(level < self.tree.len());
        let len = self.tree[level].len();
        debug_assert!(len > 0);
        debug_assert!(from < len);
        if len - from == 1 {
            return self.update_branch(level, pushed)
        }
        let mut hashes = self.tree[level][from - from % 2 ..].chunks(2)
            .map(|chunk| A::eval_hash(&(&chunk[0], &chunk[chunk.len() - 1])))
            .collect::<Vec<_>>()
            .into_iter();

        let layer_is_last = self.tree.len() == level + 1;
        let next_level = level + 1;

        if layer_is_last || pushed && from % 2 == 0 {
            return self.push_hashes_bulk(next_level, hashes);
        }
        let next_len = self.tree[next_level].len();
        debug_assert!(next_len > 0);
        self.tree[next_level][next_len - 1] = hashes.next().unwrap();
        self.tree[next_level].extend(hashes);
        self.update_branch_bulk(next_level, from / 2, false)
    }

    /// Returns the root hash, or None if tree is empty.
    pub fn get_root(&self) -> Option<&A::Value> {
        if self.tree.is_empty() {
            return None;
        }
        let last_level = self.tree.len() - 1;
        let last_index = self.tree[last_level].len() - 1;
        self.tree[last_level].get(last_index)
    }
}


#[cfg(test)]
mod tests {
    use super::MerkleTreeSimple;
    use fun::double::DoubleHash;
    use fun::sha256::Sha256;
    use fun::sha256::Sha256Value;
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

    fn cmp_proof(a: &[&str], b: &[&Sha256Value]) -> bool {
        a.len() == b.len() &&
            a.iter().cloned().map(sha256)
                .zip(b.iter())
                .all(|(x, &&y)| x == y)
    }

    fn sample_rw_tree() -> MerkleTreeSimple<&'static [u8], DoubleHash<Sha256>> {
        let mut tree = MerkleTreeSimple::default();
        tree.push(DATA[0]);
        tree.push(DATA[1]);
        tree.push(DATA[2]);
        tree
    }

    #[test]
    fn merkle_tree_simple_works() {
        let a = sample_rw_tree();
        //println!("{:?}", a);
        // println!("{:?}", tree.root());
        assert_eq!(a.get_root().unwrap(), &sha256(H20));
        assert!(a.check_data().is_ok());
        assert!(a.check_tree().is_ok());
        assert!(cmp_proof(&[H00, H10, H20], &a.audit_proof(0).unwrap()));
        assert!(cmp_proof(&[H01, H10, H20], &a.audit_proof(1).unwrap()));
        assert!(cmp_proof(&[H02, H11, H20], &a.audit_proof(2).unwrap()));

        let mut b = sample_rw_tree();
        b.clear();
        b.extend([DATA[0], DATA[1], DATA[2]].into_iter().cloned());
        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());

        b.clear();
        b.extend([DATA[0], DATA[1]].into_iter().cloned());
        b.extend([DATA[2]].into_iter().cloned());
        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());

        b.clear();
        b.extend([DATA[0]].into_iter().cloned());
        b.extend([DATA[1], DATA[2]].into_iter().cloned());
        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());

        b.clear();
        b.extend([DATA[0]].into_iter().cloned());
        b.extend([DATA[1]].into_iter().cloned());
        b.extend([DATA[2]].into_iter().cloned());
        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());

        b.clear();
        assert!(b.get_root().is_none());
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
        tree.data[1] = b"666";
        assert!(tree.check_data().is_err());
        assert!(cmp_proof(&[H00, H10, H20], &tree.audit_proof(0).unwrap()));
        assert!(tree.audit_proof(1).is_err());
        assert!(cmp_proof(&[H02, H11, H20], &tree.audit_proof(2).unwrap()));

        let mut tree = sample_rw_tree();
        {
            // Damage tree
            let layers = &mut tree.tree;
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

        let a = sample_rw_tree();
        let mut b = sample_rw_tree();
        b.rebuild();

        // println!("\n TREE a:\n\n{:?}", a);
        // println!("\n TREE b:\n\n{:?}", b);

        assert_eq!(a.get_root().unwrap(), b.get_root().unwrap());
    }
}
