use fun::abc::*;


pub trait DataStorage {
    type Block: DataBlock;
    type StorageError;

    fn get(&self, index: usize) -> Option<&Self::Block>;
    // TODO make much sound interface
    fn range<'s: 'i, 'i>(&'s self, from: usize, to: usize) -> Box<Iterator<Item=&'s Self::Block> + 'i>;
    fn push(&mut self, data: Self::Block) -> Result<(), Self::StorageError>;
}

pub trait DataBlock: MTHash {

}
