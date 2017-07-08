use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

use abc::*;


#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultHash();

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultHashValue(pub u64);

#[derive(Debug, Default, Clone)]
pub struct DefaultHashHasher {
    hasher: DefaultHasher,
}


impl MTHashFunction for DefaultHash {
    type Value = DefaultHashValue;
    type Hasher = DefaultHashHasher;
}

impl MTHasher for DefaultHashHasher {
    type Out = DefaultHashValue;

    fn new() -> Self {
        DefaultHashHasher::default()
    }

    fn write(&mut self, msg: &[u8]) {
        self.hasher.write(msg)
    }

    fn finish(self) -> Self::Out {
        DefaultHashValue(self.hasher.finish())
    }
}

impl MTValue for DefaultHashValue {

}

impl MTHashable for DefaultHashValue {
    fn hash<H: MTHasher>(&self, state: &mut H) {
        let buf: [u8; 8] = unsafe {
            ::std::mem::transmute(self.0.to_le())
        };
        state.write(&buf)
    }
}
