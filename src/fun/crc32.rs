use std::fmt;

use crc::crc32::Hasher32;
use crc::crc32::Digest;
use crc::crc32::IEEE;
use crc::crc32::CASTAGNOLI;
use crc::crc32::KOOPMAN;

use prelude::*;
use util::fmt_slice2hex;


// -------------------------------------------------------------------------------------------------


#[derive(Debug, Default, Clone, Copy)]
pub struct Crc32Ieee();

impl MTAlgorithm for Crc32Ieee {
    type Value = Crc32Value;
    type Context = Crc32IeeeContext;
}


// -------------------------------------------------------------------------------------------------


impl MTContext for Crc32IeeeContext {
    type Out = Crc32Value;

    fn new() -> Self {
        Crc32IeeeContext::default()
    }

    fn update(&mut self, msg: &[u8]) {
        self.context.write(msg)
    }

    fn finish(self) -> Self::Out {
        let digest = self.context.sum32();
        Crc32Value(unsafe { ::std::mem::transmute(digest.to_be()) })
    }
}

//#[derive(Clone)]
pub struct Crc32IeeeContext {
    context: Digest,
}

impl Default for Crc32IeeeContext {
    fn default() -> Self {
        Crc32IeeeContext {
            context: Digest::new(IEEE),
        }
    }
}

impl fmt::Debug for Crc32IeeeContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mt::fun::Crc32IeeeContext{{context: crc::crc32::Digest}}")
    }
}


// -------------------------------------------------------------------------------------------------


#[derive(Debug, Default, Clone, Copy)]
pub struct Crc32Castagnoli();

impl MTAlgorithm for Crc32Castagnoli {
    type Value = Crc32Value;
    type Context = Crc32CastagnoliContext;
}


// -------------------------------------------------------------------------------------------------


impl MTContext for Crc32CastagnoliContext {
    type Out = Crc32Value;

    fn new() -> Self {
        Crc32CastagnoliContext::default()
    }

    fn update(&mut self, msg: &[u8]) {
        self.context.write(msg)
    }

    fn finish(self) -> Self::Out {
        let digest = self.context.sum32();
        Crc32Value(unsafe { ::std::mem::transmute(digest.to_be()) })
    }
}

//#[derive(Clone)]
pub struct Crc32CastagnoliContext {
    context: Digest,
}

impl Default for Crc32CastagnoliContext {
    fn default() -> Self {
        Crc32CastagnoliContext {
            context: Digest::new(CASTAGNOLI),
        }
    }
}

impl fmt::Debug for Crc32CastagnoliContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mt::fun::Crc32CastagnoliContext{{context: crc::crc32::Digest}}")
    }
}


// -------------------------------------------------------------------------------------------------


#[derive(Debug, Default, Clone, Copy)]
pub struct Crc32Koopman();

impl MTAlgorithm for Crc32Koopman {
    type Value = Crc32Value;
    type Context = Crc32KoopmanContext;
}


// -------------------------------------------------------------------------------------------------


impl MTContext for Crc32KoopmanContext {
    type Out = Crc32Value;

    fn new() -> Self {
        Crc32KoopmanContext::default()
    }

    fn update(&mut self, msg: &[u8]) {
        self.context.write(msg)
    }

    fn finish(self) -> Self::Out {
        let digest = self.context.sum32();
        Crc32Value(unsafe { ::std::mem::transmute(digest.to_be()) })
    }
}

//#[derive(Clone)]
pub struct Crc32KoopmanContext {
    context: Digest,
}

impl Default for Crc32KoopmanContext {
    fn default() -> Self {
        Crc32KoopmanContext {
            context: Digest::new(KOOPMAN),
        }
    }
}

impl fmt::Debug for Crc32KoopmanContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mt::fun::Crc32KoopmanContext{{context: crc::crc32::Digest}}")
    }
}


// -------------------------------------------------------------------------------------------------


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Crc32Value(pub [u8; 4]);

impl fmt::Debug for Crc32Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CRC32:")?;
        fmt_slice2hex(f, &self.0[..])
    }
}

impl MTValue for Crc32Value {

}

impl MTHash for Crc32Value {
    fn hash<H: MTContext>(&self, state: &mut H) {
        state.update(self.0.as_ref())
    }
}


#[cfg(test)]
mod tests {
    use abc::MTAlgorithm;
    use super::Crc32Ieee;
    use super::Crc32Castagnoli;
    use super::Crc32Koopman;

    #[test]
    fn crc32_works() {
        let result = Crc32Ieee::eval_hash(&b"123".as_ref());
        let as_string = format!("{:?}", result);
        let sample = "CRC32:884863d2";
        assert_eq!(as_string, sample);

        let result = Crc32Ieee::eval_hash(&result);
        let as_string = format!("{:?}", result);
        let sample = "CRC32:512d162c";
        assert_eq!(as_string, sample);

        let result = Crc32Castagnoli::eval_hash(&b"123".as_ref());
        let as_string = format!("{:?}", result);
        let sample = "CRC32:107b2fb2";
        assert_eq!(as_string, sample);

        let result = Crc32Castagnoli::eval_hash(&result);
        let as_string = format!("{:?}", result);
        let sample = "CRC32:ee23e9ca";
        assert_eq!(as_string, sample);

        let result = Crc32Koopman::eval_hash(&b"123".as_ref());
        let as_string = format!("{:?}", result);
        let sample = "CRC32:6bd5eae9";
        assert_eq!(as_string, sample);

        let result = Crc32Koopman::eval_hash(&result);
        let as_string = format!("{:?}", result);
        let sample = "CRC32:e05b34cd";
        assert_eq!(as_string, sample);
    }
}
