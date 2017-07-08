pub trait MTHasher {
    type Out: MTValue;
    fn new() -> Self;
    fn write(&mut self, bytes: &[u8]);
    fn finish(self) -> Self::Out;
}

pub trait MTValue: MTHashable {
    //fn as_bytes(&self) -> &[u8];
}

pub trait MTHashable {
    fn hash<H: MTHasher>(&self, state: &mut H);

    fn hash_slice<H: MTHasher>(data: &[Self], state: &mut H)
        where Self: Sized
    {
        for piece in data {
            piece.hash(state);
        }
    }
}

pub trait MTHashFunction {
    type Value: MTValue;
    type Hasher: MTHasher<Out=Self::Value>;

    fn eval_hash<H>(data: &H) -> Self::Value where H: MTHashable {
        let mut hasher = Self::Hasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    //fn hash<D: Hashable>(data: D) -> Self;
}


impl <'a> MTHashable for &'a [u8] {
    fn hash<S: MTHasher>(&self, state: &mut S) {
        state.write(self)
    }
}

impl <'a, H> MTHashable for &'a H where H: MTHashable {
    fn hash<S: MTHasher>(&self, state: &mut S) {
        (*self).hash(state)
    }
}

impl <'a, H> MTHashable for &'a [H] where H: MTHashable {
    fn hash<S: MTHasher>(&self, state: &mut S) {
        H::hash_slice(self, state)
    }
}

impl <H> MTHashable for (H, H) where H: MTHashable {
    fn hash<S: MTHasher>(&self, state: &mut S) {
        H::hash(&self.0, state);
        H::hash(&self.1, state);
    }
}


// With #![feature(specialization)]
// impl <T> MTHasher for T where T: Default {
//     default fn new() -> Self {
//         Default::default()
//     }
// }