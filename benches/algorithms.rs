#![feature(test)]

#[macro_use]
extern crate lazy_static;
extern crate mt;
extern crate test;

use std::mem;

use test::Bencher;

use mt::data_storage::memory::MemoryReadonlyDataStorage;
use mt::data_storage::memory::MemoryDataStorage;
use mt::tree_storage::memory::MemoryTreeStorage;
use mt::merkle_tree::MerkleTree;
use mt::fun::defaulthash::DefaultHash;
use mt::fun::crc32::Crc32Ieee;
use mt::fun::crc32::Crc32Castagnoli;
use mt::fun::crc32::Crc32Koopman;
use mt::fun::sha256::Sha256;
use mt::fun::double::DoubleHash;


static N: u32 = 100;

lazy_static! {
    static ref DATA_1000_X4: Vec<[u8; 4]> = (0..N)
        .map(|x| unsafe { mem::transmute(x.to_be()) })
        .collect();

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
fn n_x_0004_bulk_crc32_ieee(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Ieee>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_step_by_step_crc32_ieee(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Ieee>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_bulk_crc32_castagnoli(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Castagnoli>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_step_by_step_crc32_castagnoli(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Castagnoli>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_bulk_crc32_koopman(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Koopman>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_step_by_step_crc32_koopman(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Koopman>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_bulk_sha256(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<Sha256>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_step_by_step_sha256(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Sha256>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_bulk_sha256_double(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4.iter().map(|v| &v[..]).collect();
    b.iter(|| {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<Sha256>>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_step_by_step_sha256_double(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<Sha256>>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_bulk_rust_siphash(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<DefaultHash>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_step_by_step_rust_siphash(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DefaultHash>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_bulk_rust_siphash_double(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4.iter().map(|v| &v[..]).collect();
    b.iter(|| {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<DefaultHash>>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_0004_step_by_step_rust_siphash_double(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<DefaultHash>>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}



// =================================================================================================


#[bench]
fn n_x_4096_bulk_crc32_ieee(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Ieee>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_step_by_step_crc32_ieee(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Ieee>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_crc32_castagnoli(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Castagnoli>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_step_by_step_crc32_castagnoli(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Castagnoli>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_crc32_koopman(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Koopman>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_step_by_step_crc32_koopman(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Crc32Koopman>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_sha256(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<Sha256>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_step_by_step_sha256(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<Sha256>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_sha256_double(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(|| {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<Sha256>>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_step_by_step_sha256_double(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<Sha256>>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_rust_siphash(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(move || {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<DefaultHash>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_step_by_step_rust_siphash(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DefaultHash>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_bulk_rust_siphash_double(b: &mut Bencher) {
    let data: Vec<&'static [u8]> = DATA_1000_X4096.iter().map(|v| &v[..]).collect();
    b.iter(|| {
        let mt: MerkleTree<MemoryReadonlyDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<DefaultHash>>>;
        mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::new(data.as_slice()), Default::default()).unwrap();
        test::black_box(mt);
    });
}


#[bench]
fn n_x_4096_step_by_step_rust_siphash_double(b: &mut Bencher) {
    b.iter(|| {
        let mut mt: MerkleTree<MemoryDataStorage<&'static [u8]>, MemoryTreeStorage<DoubleHash<DefaultHash>>>;
        mt = MerkleTree::default();
        for x in DATA_1000_X4096.iter() {
            mt.push(x.as_ref()).unwrap();
        }
        test::black_box(mt);
    });
}
