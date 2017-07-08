use super::abc::*;


pub struct MemoryStorage<V> where V: DataBlock {
    data: Vec<V>
}

impl <V> DataStorage for MemoryStorage<V> where V: DataBlock {
    type Block = V;
    type StorageError = ();

    fn get(&self, index: usize) -> Option<&Self::Block> {
        self.data.get(index)
    }

    fn range<'s: 'i, 'i>(&'s self, from: usize, to: usize) -> Box<Iterator<Item=&'s Self::Block> + 'i> {
        Box::new(self.data[from..to].iter())
    }

    fn push(&mut self, data: Self::Block) -> Result<(), Self::StorageError> {
        self.data.push(data);
        Ok(())
    }
}
