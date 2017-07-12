use std::fmt;
use mt::prelude::*;


pub struct Chunk4096(pub [u8; 4096]);

impl Copy for Chunk4096 {}

impl Clone for Chunk4096 {
    fn clone(&self) -> Self {
        Chunk4096( unsafe { ::std::ptr::read(&self.0) } )
    }
}

impl Chunk4096 {
    pub fn new() -> Self {
        Chunk4096 ( [0; 4096] )
    }
}

impl Default for Chunk4096 {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Chunk4096 {
    fn eq(&self, other: &Chunk4096) -> bool {
        self.0[..].iter()
            .zip(other.0.iter())
            .all(|(a, b)| a == b)
    }
}

impl Eq for Chunk4096 {}

impl fmt::Debug for Chunk4096 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Chunk4096")
    }
}

impl MTHash for Chunk4096 {
    fn hash<H: MTContext>(&self, state: &mut H) {
        state.update(self.0.as_ref())
    }
}
