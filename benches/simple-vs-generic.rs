#![feature(test)]

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


mod data_chunk;
use data_chunk::Chunk4096;


fn make_data(n: u32) -> Vec<Chunk4096> {
    (0..n)
        .map(|x| {
            let mut data = Chunk4096::new();
            for (y, v) in data.0.iter_mut().enumerate() {
                *v = x as u8 ^ y as u8;
            }
            data
        })
        .collect()
}

macro_rules! bulk_generic {
    ($n: expr) => {
        #[bench]
        fn bulk_generic(b: &mut Bencher) {
            let data = make_data($n);
            b.iter(|| {
                let mt: MerkleTree<MemoryDataStorage<Chunk4096>, MemoryTreeStorage<Type>>;
                mt = MerkleTree::new_and_rebuild(MemoryDataStorage::with_data(data.clone()), Default::default()).unwrap();
                test::black_box(mt);
            });
        }
    };
}

macro_rules! bulk_generic_readonly {
    ($n: expr) => {
        #[bench]
        fn bulk_generic_readonly(b: &mut Bencher) {
            let data = make_data($n);
            b.iter(|| {
                let mt: MerkleTree<MemoryReadonlyDataStorage<Chunk4096>, MemoryTreeStorage<Type>>;
                mt = MerkleTree::new_and_rebuild(MemoryReadonlyDataStorage::with_data(data.clone()), Default::default()).unwrap();
                test::black_box(mt);
            });
        }
    };
}

macro_rules! bulk_simple {
    ($n: expr) => {
        #[bench]
        fn bulk_simple(b: &mut Bencher) {
            let data = make_data($n);
            b.iter(|| {
                let mt: MerkleTreeSimple<Chunk4096, Type>;
                mt = MerkleTreeSimple::new_and_rebuild(data.clone());
                test::black_box(mt);
            });
        }
    };
}

macro_rules! step_by_step_generic {
    ($n: expr) => {
        #[bench]
        fn step_by_step_generic(b: &mut Bencher) {
            let data = make_data($n);
            b.iter(|| {
                let mut mt: MerkleTree<MemoryDataStorage<Chunk4096>, MemoryTreeStorage<Type>>;
                mt = MerkleTree::default();
                for &x in data.iter() {
                    mt.push(x).unwrap();
                }
                test::black_box(mt);
            });
        }
    };
}

macro_rules! step_by_step_simple {
    ($n: expr) => {
        #[bench]
        fn step_by_step_simple(b: &mut Bencher) {
            let data = make_data($n);
            b.iter(|| {
                let mut mt: MerkleTreeSimple<Chunk4096, Type>;
                mt = MerkleTreeSimple::default();
                for &x in data.iter() {
                    mt.push(x);
                }
                test::black_box(mt);
            });
        }
    };
}

macro_rules! step_bulk_generic {
    ($n: expr, $name: ident, $e: expr) => {
        mod $name {
            use super::*;

            #[bench]
            fn generic(b: &mut Bencher) {
                let data = make_data($n);
                b.iter(|| {
                    let mut mt: MerkleTree<MemoryDataStorage<Chunk4096>, MemoryTreeStorage<Type>>;
                    mt = MerkleTree::default();
                    for x in data.chunks($e) {
                        mt.extend(x.iter().cloned().map(Ok)).unwrap();
                    }
                    test::black_box(mt);
                });
            }

            #[bench]
            fn simple(b: &mut Bencher) {
                let data = make_data($n);
                b.iter(|| {
                    let mut mt: MerkleTreeSimple<Chunk4096, Type>;
                    mt = MerkleTreeSimple::default();
                    for x in data.chunks($e) {
                        mt.extend(x.iter().cloned());
                    }
                    test::black_box(mt);
                });
            }
        }
    };
    ($n: expr) => {
        mod step_bulk {
            use super::*;

            step_bulk_generic!($n, x1e1, 1);
            step_bulk_generic!($n, x1e2, 10);
            step_bulk_generic!($n, x1e3, 100);
            step_bulk_generic!($n, x1e4, 1000);
        }
    };
}

macro_rules! bench_n {
    ( $name: ident, $n: expr ) => {

        #[allow(non_snake_case)]
        mod $name {
            use super::*;

            mod x_4096 {
                use super::*;

                bulk_generic!($n);
                bulk_generic_readonly!($n);
                bulk_simple!($n);
                step_by_step_generic!($n);
                step_by_step_simple!($n);
                step_bulk_generic!($n);
            }
        }
    };
}

macro_rules! bench {
    ( $name: ident, $fun: ty ) => {

mod $name {
    use super::*;
    type Type = $fun;

    //bench_n!(n1e1, 1);
    //bench_n!(n1e2, 10);
    bench_n!(n1e3, 100);
    bench_n!(n1e4, 1000);
    bench_n!(n1e5, 10000);
    //bench_n!(n1e6, 100000);
    //bench_n!(n1e7, 1000000);
}
    };
}

bench!(crc32, Crc32Ieee);
bench!(sha256, Sha256);
bench!(rust_siphash, DefaultHash);
