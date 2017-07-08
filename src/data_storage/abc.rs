use std::fmt;

use fun::abc::*;


pub trait DataStorage: fmt::Debug {
    type Block: DataBlock;
    type StorageError: fmt::Debug;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, index: usize) -> Option<&Self::Block>;
    fn iter<'s: 'i, 'i>(&'s self) -> Box<Iterator<Item=&'s Self::Block> + 'i>;
    // FIXME make a more reliable interface
    fn range<'s: 'i, 'i>(&'s self, from: usize, to: usize) -> Box<Iterator<Item=&'s Self::Block> + 'i>;
    fn push(&mut self, data: Self::Block) -> Result<(), Self::StorageError>;
}

pub trait DataBlock: MTHash {

}

// FIXME stub
impl <T> DataBlock for T where T: MTHash {}
