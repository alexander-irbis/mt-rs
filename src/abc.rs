pub trait MTContext {
    type Out: MTValue;
    fn new() -> Self;
    fn update(&mut self, msg: &[u8]);
    fn finish(self) -> Self::Out;
}

pub trait MTValue: MTHash {
    //fn as_bytes(&self) -> &[u8];
}

pub trait MTHash {
    fn hash<H: MTContext>(&self, state: &mut H);

    fn hash_slice<H: MTContext>(data: &[Self], state: &mut H)
        where Self: Sized
    {
        for piece in data {
            piece.hash(state);
        }
    }
}

pub trait MTAlgorithm {
    type Value: MTValue;
    type Context: MTContext<Out=Self::Value>;

    fn eval_hash<H>(data: &H) -> Self::Value where H: MTHash {
        let mut context = Self::Context::new();
        data.hash(&mut context);
        context.finish()
    }

    //fn hash<D: Hashable>(data: D) -> Self;
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
