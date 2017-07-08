use std::fmt;

use ring::digest::Context;
use ring::digest::Digest;
use ring::digest::SHA256;

use abc::*;


#[derive(Debug, Default, Clone, Copy)]
pub struct Sha256();

#[derive(Debug, Clone, Copy)]
pub struct Sha256Value(pub Digest);

#[derive(Clone)]
pub struct Sha256Context {
    context: Context,
}

impl Default for Sha256Context {
    fn default() -> Self {
        Sha256Context {
            context: Context::new(&SHA256),
        }
    }
}

impl fmt::Debug for Sha256Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mt::fun::Sha256Hasher{{context: ring::digest::Context}}")
    }
}


impl MTAlgorithm for Sha256 {
    type Value = Sha256Value;
    type Context = Sha256Context;
}

impl MTContext for Sha256Context {
    type Out = Sha256Value;

    fn new() -> Self {
        Sha256Context::default()
    }

    fn update(&mut self, msg: &[u8]) {
        self.context.update(msg)
    }

    fn finish(self) -> Self::Out {
        Sha256Value(self.context.finish())
    }
}

impl MTValue for Sha256Value {

}

impl MTHash for Sha256Value {
    fn hash<H: MTContext>(&self, state: &mut H) {
        state.update(self.0.as_ref())
    }
}
