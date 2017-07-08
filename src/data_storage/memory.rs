use std::fmt;

use super::abc::*;


pub struct MemoryDataStorage<V> where V: DataBlock {
    data: Vec<V>
}

impl <V> Default for MemoryDataStorage<V> where V: DataBlock {
    fn default() -> Self {
        MemoryDataStorage {
            data: Vec::new()
        }
    }
}

impl<V> MemoryDataStorage<V> where V: DataBlock {
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut Vec<V> {
        &mut self.data
    }
}

impl <V> fmt::Debug for MemoryDataStorage<V> where V: DataBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryDataStorage(len={})", self.len())
    }
}

impl <V> DataStorage for MemoryDataStorage<V> where V: DataBlock {
    type Block = V;
    type StorageError = ();

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn get(&self, index: usize) -> Option<&Self::Block> {
        self.data.get(index)
    }

    fn iter<'s: 'i, 'i>(&'s self) -> Box<Iterator<Item=&'s Self::Block> + 'i> {
        Box::new(self.data.iter())
    }

    fn range<'s: 'i, 'i>(&'s self, from: usize, to: usize) -> Box<Iterator<Item=&'s Self::Block> + 'i> {
        Box::new(self.data[from..to].iter())
    }

    fn push(&mut self, data: Self::Block) -> Result<(), Self::StorageError> {
        self.data.push(data);
        Ok(())
    }
}
