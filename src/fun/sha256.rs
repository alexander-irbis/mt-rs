use std::fmt;

use ring::digest::Context;
use ring::digest::Digest;
use ring::digest::SHA256;

use abc::*;


#[derive(Debug, Default, Clone, Copy)]
pub struct Sha256Hash();

#[derive(Debug, Clone, Copy)]
pub struct Sha256Value(pub Digest);

#[derive(Clone)]
pub struct Sha256Hasher {
    context: Context,
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Sha256Hasher {
            context: Context::new(&SHA256),
        }
    }
}

impl fmt::Debug for Sha256Hasher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mt::fun::Sha256Hasher{{context: ring::digest::Context}}")
    }
}


impl MTHashFunction for Sha256Hash {
    type Value = Sha256Value;
    type Hasher = Sha256Hasher;
}

impl MTHasher for Sha256Hasher {
    type Out = Sha256Value;

    fn new() -> Self {
        Sha256Hasher::default()
    }

    fn write(&mut self, msg: &[u8]) {
        self.context.update(msg)
    }

    fn finish(self) -> Self::Out {
        Sha256Value(self.context.finish())
    }
}

impl MTValue for Sha256Value {

}

impl MTHashable for Sha256Value {
    fn hash<H: MTHasher>(&self, state: &mut H) {
        state.write(self.0.as_ref())
    }
}
