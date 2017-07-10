use std::fmt;

use prelude::*;


/// Static data (for example, on read-only media).
/// Can only be checked
pub trait DataStorageReadonly: fmt::Debug {
    type Block: DataBlock;

    fn len(&self) -> Result<usize>;
    fn is_empty(&self) -> Result<bool> {
        self.len().map(|len| len == 0)
    }
    fn get(&self, index: usize) -> Result<Self::Block>;
    fn iter<'s: 'i, 'i>(&'s self) -> Result<Box<Iterator<Item=Result<Self::Block>> + 'i>> {
        Ok(Box::new((0 .. self.len()?)
            .map( move |index| self.get(index) )
        ))
    }
    fn range<'s: 'i, 'i>(&'s self, from: usize, to: usize) -> Result<Box<Iterator<Item=Result<Self::Block>> + 'i>> {
        let len = self.len()?;
        if to > len {
            // FIXME error
            panic!("`to` ({}) greater then `len` ({})", to, len);
        }
        if from > to {
            // FIXME error
            panic!("`from` ({}) greater then `to` ({})", from, to);
        }
        Ok(Box::new((from .. to)
            .map( move |index| self.get(index) )
        ))
    }

    /// Writable data storage have to override this method
    fn is_writeable(&self) -> bool {
        false
    }
}


pub trait DataStorage: DataStorageReadonly {
    fn push(&mut self, data: Self::Block) -> Result<()>;
}

pub trait DataBlock: MTHash {

}

// FIXME stub
impl <T> DataBlock for T where T: MTHash {}
