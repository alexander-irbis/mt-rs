use std::borrow::Cow;
use std::fmt;
use std::ops;

use prelude::*;


/// An inmemory storage for data values
/// May be just a reference to the real data storage
pub struct MemoryReadonlyDataStorage<'v, V> where V: MTHash + 'v {
    data: Cow<'v, [V]>,
}

impl <'v, V> MemoryReadonlyDataStorage<'v, V> where V: MTHash + 'v {
    /// Creates an instance, representing `data`
    pub fn with_data<D: Into<Cow<'v, [V]>>>(data: D) -> Self {
        MemoryReadonlyDataStorage { data: data.into() }
    }
}

impl <'v, V> fmt::Debug for MemoryReadonlyDataStorage<'v, V> where V: MTHash + 'v {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryDataStorage(len={})", self.len().map_err(|_| fmt::Error)?)
    }
}

impl <'v, V> DataStorageReadonly for MemoryReadonlyDataStorage<'v, V> where V: MTHash + 'v {
    type DataValue = V;

    fn len(&self) -> Result<usize> {
        Ok(self.data.len())
    }

    fn is_empty(&self) -> Result<bool> {
        Ok(self.data.is_empty())
    }

    fn get(&self, index: usize) -> Result<Self::DataValue> {
        self.data.get(index).cloned().ok_or(INDEX_IS_OUT_OF_BOUNDS)
    }

    fn iter<'s: 'i, 'i>(&'s self) -> Result<Box<Iterator<Item=Result<Self::DataValue>> + 'i>> {
        Ok(Box::new(self.data.iter().cloned().map(Ok)))
    }

    fn range<'s: 'i, 'i, R: Into<ops::Range<usize>>>(&'s self, range: R) -> Result<Box<Iterator<Item=Result<Self::DataValue>> + 'i>> {
        let range = range.into();
        self.check_range(&range)?;
        Ok(Box::new(self.data[range].iter().cloned().map(Ok)))
    }
}


/// A writable inmemory data storage
pub struct MemoryDataStorage<V> where V: MTHash {
    data: Vec<V>,
    is_writable: bool,
}

impl <V> Default for MemoryDataStorage<V> where V: MTHash {
    fn default() -> Self {
        MemoryDataStorage::new()
    }
}

impl<V> MemoryDataStorage<V> where V: MTHash {
    /// Creates a new empty instance
    pub fn new() -> Self {
        MemoryDataStorage {
            data: Vec::new(),
            is_writable: true,
        }
    }

    /// Creates an instance, filled with `data`
    pub fn with_data<VV: Into<Vec<V>>>(data: VV) -> Self {
        MemoryDataStorage {
            data: data.into(),
            is_writable: true,
        }
    }

    /// sets whether data storage can accept new values
    pub fn set_writable(&mut self, is_writable: bool) {
        self.is_writable = is_writable;
    }

    /// For testing purposes only
    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut Vec<V> {
        &mut self.data
    }
}

impl <V> fmt::Debug for MemoryDataStorage<V> where V: MTHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryDataStorage(len={})", self.data.len())
    }
}

impl <V> DataStorageReadonly for MemoryDataStorage<V> where V: MTHash {
    type DataValue = V;

    fn len(&self) -> Result<usize> {
        Ok(self.data.len())
    }

    fn is_empty(&self) -> Result<bool> {
        Ok(self.data.is_empty())
    }

    fn get(&self, index: usize) -> Result<Self::DataValue> {
        self.data.get(index).cloned().ok_or(INDEX_IS_OUT_OF_BOUNDS)
    }

    fn iter<'s: 'i, 'i>(&'s self) -> Result<Box<Iterator<Item=Result<Self::DataValue>> + 'i>> {
        Ok(Box::new(self.data.iter().cloned().map(Ok)))
    }

    fn range<'s: 'i, 'i, R: Into<ops::Range<usize>>>(&'s self, range: R) -> Result<Box<Iterator<Item=Result<Self::DataValue>> + 'i>> {
        let range = range.into();
        self.check_range(&range)?;
        Ok(Box::new(self.data[range].iter().cloned().map(Ok)))
    }

    fn is_writeable(&self) -> bool {
        self.is_writable
    }
}

impl <V> DataStorage for MemoryDataStorage<V> where V: MTHash {
    /// Clears all data
    fn clear(&mut self) -> Result<()> {
        Ok(self.data.clear())
    }

    fn push(&mut self, data: Self::DataValue) -> Result<()> {
        if self.is_writeable() {
            self.data.push(data);
            Ok(())
        } else {
            Err(Error::new_ro("The data storage is in read-only mode"))
        }
    }

    fn extend<DD: IntoIterator<Item=Result<Self::DataValue>>>(&mut self, data: DD) -> Result<()> {
        for v in data.into_iter() {
            self.data.push(v?);
        }
        Ok(())
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
        assert!(ds.len().unwrap() == 2);
    }

    #[test]
    fn memory_readonly_data_storage() {
        let data: [&[u8]; 3] = [DATA[0], DATA[1], DATA[2]];
        let ds = MemoryReadonlyDataStorage::with_data(&data[..]);
        assert!(ds.len().unwrap() == 3);
        assert!(!ds.is_writeable());
    }
}
