use std::marker::PhantomData;
use abc::*;


/// Doubles hash
///
/// ```ignore
/// dhash<sha256>(x) -> sha256(sha256(x))
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct DoubleHash<H> where H: MTAlgorithm {
    marker: PhantomData<H>,
}

impl <H> DoubleHash<H> where H: MTAlgorithm {
    pub fn new() -> Self {
        DoubleHash {
            marker: PhantomData,
        }
    }
}

impl <H> MTAlgorithm for DoubleHash<H> where H: MTAlgorithm {
    type Value = H::Value;
    type Context = H::Context;

    fn eval_hash<D>(data: &D) -> Self::Value where D: MTHash {
        let mut context = Self::Context::new();
        data.hash(&mut context);
        let value = context.finish();

        let mut context = Self::Context::new();
        value.hash(&mut context);
        context.finish()
    }
}

