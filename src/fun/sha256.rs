use std::fmt;

use ring::digest::Context;
use ring::digest::SHA256;

use abc::*;
use util::fmt_slice2hex;


#[derive(Debug, Default, Clone, Copy)]
pub struct Sha256();

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Sha256Value(pub [u8; 32]);

#[derive(Clone)]
pub struct Sha256Context {
    context: Context,
}

impl fmt::Debug for Sha256Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SHA256:")?;
        fmt_slice2hex(f, &self.0[..])
    }
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
        let mut value: [u8; 32] = Default::default();
        let digest = self.context.finish();
        value.clone_from_slice(digest.as_ref());
        Sha256Value(value)
    }
}

impl MTValue for Sha256Value {

}

impl MTHash for Sha256Value {
    fn hash<H: MTContext>(&self, state: &mut H) {
        state.update(self.0.as_ref())
    }
}


#[cfg(test)]
mod tests {
    use abc::MTAlgorithm;
    use super::Sha256;

    #[test]
    fn sha256_works() {
        let result = Sha256::eval_hash(&b"123".as_ref());
        let as_string = format!("{:?}", result);
        let sample = "SHA256:a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3";
        assert_eq!(as_string, sample);

        let result = Sha256::eval_hash(&result);
        let as_string = format!("{:?}", result);
        let sample = "SHA256:5a77d1e9612d350b3734f6282259b7ff0a3f87d62cfef5f35e91a5604c0490a3";
        assert_eq!(as_string, sample);
    }
}
