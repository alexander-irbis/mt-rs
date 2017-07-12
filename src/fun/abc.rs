use std::fmt;


/// Represents a state of hashing
pub trait MTContext {
    type Out: MTHash;
    fn new() -> Self;
    fn update(&mut self, msg: &[u8]);
    fn finish(self) -> Self::Out;
}

/// Represants a hashable value
/// (hashes should be hashable too)
pub trait MTHash: Eq + Clone + fmt::Debug {
    fn hash<H: MTContext>(&self, state: &mut H);

    fn hash_slice<H: MTContext>(data: &[Self], state: &mut H)
        where Self: Sized
    {
        for piece in data {
            piece.hash(state);
        }
    }
}

/// Represents a hashing algorithm
pub trait MTAlgorithm {
    type Value: MTHash;
    type Context: MTContext<Out=Self::Value>;

    fn eval_hash<H>(data: &H) -> Self::Value where H: MTHash {
        let mut context = Self::Context::new();
        data.hash(&mut context);
        context.finish()
    }
}


impl <'a> MTHash for &'a [u8] {
    fn hash<S: MTContext>(&self, state: &mut S) {
        state.update(self)
    }
}

impl <'a, H> MTHash for &'a H where H: MTHash {
    fn hash<S: MTContext>(&self, state: &mut S) {
        (*self).hash(state)
    }
}

impl <'a, H> MTHash for &'a [H] where H: MTHash {
    fn hash<S: MTContext>(&self, state: &mut S) {
        H::hash_slice(self, state)
    }
}

impl <H> MTHash for (H, H) where H: MTHash {
    fn hash<S: MTContext>(&self, state: &mut S) {
        H::hash(&self.0, state);
        H::hash(&self.1, state);
    }
}
