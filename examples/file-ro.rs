extern crate tempfile;
extern crate mt;

mod fs;

use std::fs::File;
use std::io::Write;

use mt::abc::TreeStorage;
use mt::tree_storage::memory::MemoryTreeStorage;
use mt::merkle_tree::MerkleTree;
use mt::fun::sha256::Sha256;
use mt::fun::double::DoubleHash;
use fs::data_storage::*;

fn main() {
    let mut tmpfile: File = tempfile::tempfile().unwrap();
    write!(tmpfile, "Hello World!").unwrap();

    let storage = ChunkedFile::new(tmpfile).unwrap();
    let mt: MerkleTree<ChunkedFile, MemoryTreeStorage<DoubleHash<Sha256>>>;
    mt = MerkleTree::new_and_rebuild(storage, Default::default()).unwrap();
    println!("Tree root: {:?}", mt.tree().get_root().unwrap());
}