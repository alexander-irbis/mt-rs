#![feature(test)]

#[macro_use]
extern crate lazy_static;
extern crate mt;
extern crate test;

use test::Bencher;

use mt::data_storage::memory::MemoryReadonlyDataStorage;
use mt::data_storage::memory::MemoryDataStorage;
use mt::tree_storage::memory::MemoryTreeStorage;
use mt::merkle_tree::MerkleTree;
use mt::merkle_tree::MerkleTreeSimple;
use mt::fun::crc32::Crc32Ieee;
use mt::fun::defaulthash::DefaultHash;
use mt::fun::sha256::Sha256;


static N: u32 = 10000;

lazy_static! {
    static ref DATA_1000_X4096: Vec<[u8; 4096]> = (0..N)
        .map(|x| {
            let mut data = [0u8; 4096];
            for (y, v) in data.iter_mut().enumerate() {
                *v = x as u8 ^ y as u8;
            }
            data
        })
        .collect();
}


#[bench]
fn n_x_4096_bulk_crc32_generic(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Ieee>>;
        mt = MerkleTree::new_and_rebuild(MemoryDataStorage::with_data(data.clone()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_crc32_simple(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTreeSimple<&'static [u8], Crc32Ieee>;
        mt = MerkleTreeSimple::new_and_rebuild(data.clone());
        test::black_box(mt);
    });
}


#[bench] #[ignore]
fn n_x_4096_step_by_step_crc32_generic(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Ieee>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench] #[ignore]
fn n_x_4096_step_by_step_crc32_simple(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTreeSimple<&'static [u8], Crc32Ieee>;
        mt = MerkleTreeSimple::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref());
        }
        test::black_box(mt);
    });
}


// -------------------------------------------------------------------------------------------------


#[bench]
fn n_x_4096_bulk_sha256_generic(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Sha256>>;
        mt = MerkleTree::new_and_rebuild(MemoryDataStorage::with_data(data.clone()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_sha256_simple(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTreeSimple<&'static [u8], Sha256>;
        mt = MerkleTreeSimple::new_and_rebuild(data.clone());
        test::black_box(mt);
    });
}


#[bench] #[ignore]
fn n_x_4096_step_by_step_sha256_generic(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Sha256>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench] #[ignore]
fn n_x_4096_step_by_step_sha256_simple(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTreeSimple<&'static [u8], Sha256>;
        mt = MerkleTreeSimple::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref());
        }
        test::black_box(mt);
    });
}


// -------------------------------------------------------------------------------------------------


#[bench]
fn n_x_4096_bulk_rust_siphash_generic_readonly(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<DefaultHash>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_rust_siphash_generic(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DefaultHash>>;
        mt = MerkleTree::new_and_rebuild(MemoryDataStorage::with_data(data.clone()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_rust_siphash_simple(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTreeSimple<&'static [u8], DefaultHash>;
        mt = MerkleTreeSimple::new_and_rebuild(data.clone());
        test::black_box(mt);
    });
}


#[bench] #[ignore]
fn n_x_4096_step_by_step_rust_siphash_generic(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DefaultHash>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench] #[ignore]
fn n_x_4096_step_by_step_rust_siphash_simple(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTreeSimple<&'static [u8], DefaultHash>;
        mt = MerkleTreeSimple::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref());
        }
        test::black_box(mt);
    });
}
