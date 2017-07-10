use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

use prelude::*;


#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultHash();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultHashValue(pub u64);

#[derive(Debug, Default, Clone)]
pub struct DefaultHashContext {
    context: DefaultHasher,
}


impl MTAlgorithm for DefaultHash {
    type Value = DefaultHashValue;
    type Context = DefaultHashContext;
}

impl MTContext for DefaultHashContext {
    type Out = DefaultHashValue;

    fn new() -> Self {
        DefaultHashContext::default()
    }

    fn update(&mut self, msg: &[u8]) {
        self.context.write(msg)
    }

    fn finish(self) -> Self::Out {
        DefaultHashValue(self.context.finish())
    }
}

impl MTValue for DefaultHashValue {

}

impl MTHash for DefaultHashValue {
    fn hash<H: MTContext>(&self, state: &mut H) {
        let buf: [u8; 8] = unsafe {
            ::std::mem::transmute(self.0.to_le())
        };
        state.update(&buf)
    }
}
