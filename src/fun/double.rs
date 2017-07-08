use std::marker::PhantomData;
use abc::*;


/// Doubles hash
/// dhash<sha256>(x) -> sha256(sha256(x))
#[derive(Debug, Default, Clone, Copy)]
pub struct DoubleHash<H> where H: MTHashFunction {
    marker: PhantomData<H>,
}

impl <H> DoubleHash<H> where H: MTHashFunction {
    pub fn new() -> Self {
        DoubleHash {
            marker: PhantomData,
        }
    }
}

impl <H> MTHashFunction for DoubleHash<H> where H: MTHashFunction {
    type Value = H::Value;
    type Hasher = H::Hasher;

    fn eval_hash<D>(data: &D) -> Self::Value where D: MTHashable {
        let mut hasher = Self::Hasher::new();
        data.hash(&mut hasher);
        let value = hasher.finish();

        let mut hasher = Self::Hasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
