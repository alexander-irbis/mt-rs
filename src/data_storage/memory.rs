use std::borrow::Cow;
use std::fmt;

use super::abc::*;


pub struct MemoryReadonlyDataStorage<'v, V> where V: DataBlock + 'v {
    data: Cow<'v, [V]>,
}

impl <'v, V> MemoryReadonlyDataStorage<'v, V> where V: DataBlock + 'v {
    pub fn new<D: Into<Cow<'v, [V]>>>(data: D) -> Self {
        MemoryReadonlyDataStorage { data: data.into() }
    }
}

impl <'v, V> fmt::Debug for MemoryReadonlyDataStorage<'v, V> where V: DataBlock + 'v {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryDataStorage(len={})", self.len())
    }
}

impl <'v, V> DataStorageReadonly for MemoryReadonlyDataStorage<'v, V> where V: DataBlock + 'v {
    type Block = V;
    type StorageError = CommonDataStorageError;

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
}


pub struct MemoryDataStorage<V> where V: DataBlock {
    data: Vec<V>,
    is_writable: bool,
}

impl <V> Default for MemoryDataStorage<V> where V: DataBlock {
    fn default() -> Self {
        MemoryDataStorage::new()
    }
}

impl<V> MemoryDataStorage<V> where V: DataBlock {
    pub fn new() -> Self {
        MemoryDataStorage {
            data: Vec::new(),
            is_writable: true,
        }
    }

    pub fn set_writable(&mut self, is_writable: bool) {
        self.is_writable = is_writable;
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

impl <V> DataStorageReadonly for MemoryDataStorage<V> where V: DataBlock {
    type Block = V;
    type StorageError = CommonDataStorageError;

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

    fn is_writeable(&self) -> bool {
        self.is_writable
    }
}

impl <V> DataStorage for MemoryDataStorage<V> where V: DataBlock {
    fn push(&mut self, data: Self::Block) -> Result<(), Self::StorageError> {
        if self.is_writeable() {
            self.data.push(data);
            Ok(())
        } else {
            Err(CommonDataStorageError::new_ro("The data storage is in read-only mode"))
        }
    }
}


#[cfg(test)]
mod tests {
    use abc::*;
    use super::MemoryDataStorage;
    use super::MemoryReadonlyDataStorage;

    static DATA: [&[u8]; 3] = [b"123", b"321", b"555"];

    #[test]
    fn memory_data_storage_ro_rw_mode() {
        let mut ds = MemoryDataStorage::default();
        assert!(ds.push(&b"123"[..]).is_ok());
        ds.set_writable(false);
        assert!(ds.push(&b"123"[..]).is_err());
        ds.set_writable(true);
        assert!(ds.push(&b"123"[..]).is_ok());
        assert!(ds.len() == 2);
    }

    #[test]
    fn memory_readonly_data_storage() {
        let data: [&[u8]; 3] = [DATA[0], DATA[1], DATA[2]];
        let ds = MemoryReadonlyDataStorage::new(&data[..]);
        assert!(ds.len() == 3);
        assert!(!ds.is_writeable());
    }
}
