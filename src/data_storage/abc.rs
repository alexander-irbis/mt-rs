use std::fmt;
use std::io;

use fun::abc::*;


pub trait DataStorageError: fmt::Debug {
    fn is_io_error(&self) -> bool;
    fn into_io_error(self) -> Option<io::Error>;
}

#[derive(Debug)]
pub enum CommonDataStorageError {
    Io(io::Error)
}

impl DataStorageError for CommonDataStorageError {
    fn is_io_error(&self) -> bool {
        match *self {
            CommonDataStorageError::Io(_) => true,
            // _ => false,
        }
    }

    fn into_io_error(self) -> Option<io::Error> {
        match self {
            CommonDataStorageError::Io(err) => Some(err),
            // _ => None,
        }
    }
}

impl CommonDataStorageError {
    pub fn new_ro<M: Into<String>>(msg: M) -> Self {
        CommonDataStorageError::Io(io::Error::new(io::ErrorKind::PermissionDenied, msg.into()))
    }
}


/// Static data (for example, on read-only media).
/// Can only be checked
pub trait DataStorageReadonly: fmt::Debug {
    type Block: DataBlock;
    type StorageError: DataStorageError;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, index: usize) -> Option<&Self::Block>;
    fn iter<'s: 'i, 'i>(&'s self) -> Box<Iterator<Item=&'s Self::Block> + 'i>;
    // FIXME make a more reliable interface
    fn range<'s: 'i, 'i>(&'s self, from: usize, to: usize) -> Box<Iterator<Item=&'s Self::Block> + 'i>;

    /// Writable data storage have to override this method
    fn is_writeable(&self) -> bool {
        return false;
    }
}


pub trait DataStorage: DataStorageReadonly {
    fn push(&mut self, data: Self::Block) -> Result<(), Self::StorageError>;
}

pub trait DataBlock: MTHash {

}

// FIXME stub
impl <T> DataBlock for T where T: MTHash {}
