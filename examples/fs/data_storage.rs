use std::cell::RefCell;
use std::fmt;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io;
use std::io::Read;
use std::path::Path;

use mt::prelude::*;


// -------------------------------------------------------------------------------------------------


pub struct Chunk4096 {
    data: [u8; 4096],
    size: usize,
}

impl Chunk4096 {
    pub fn new() -> Self {
        Chunk4096 {
            data: [0; 4096],
            size: 0,
        }
    }

    pub fn from_stream<R: Read>(fd: &mut R) -> io::Result<Self> {
        let mut chunk = Self::default();
        let size = fd.read(&mut chunk.data)?;
        chunk.size = size;
        Ok(chunk)
    }
}

impl Default for Chunk4096 {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Chunk4096 {
    fn eq(&self, other: &Chunk4096) -> bool {
        self.data[..].iter()
            .zip(other.data.iter())
            .all(|(a, b)| a == b)
    }
}

impl Eq for Chunk4096 {}

impl Clone for Chunk4096 {
    fn clone(&self) -> Self {
        Chunk4096 {
            data: unsafe { ::std::ptr::read(&self.data) },
            size: self.size
        }
    }
}

impl fmt::Debug for Chunk4096 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Chunk4096(size={})", self.size)
    }
}

impl MTHash for Chunk4096 {
    fn hash<H: MTContext>(&self, state: &mut H) {
        state.update(&self.data[.. self.size])
    }
}


// -------------------------------------------------------------------------------------------------


pub struct ChunkedFile {
    fd: RefCell<File>,
    size: u64,
}

impl ChunkedFile {
    pub fn new(fd: File) -> io::Result<Self> {
        let size = fd.metadata()?.len();
        let fd = RefCell::new(fd);
        Ok(ChunkedFile { fd, size })
    }

    #[allow(dead_code)]
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        ChunkedFile::new(File::open(path)?)
    }
}

impl fmt::Debug for ChunkedFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ChunkedFile(len={},size={})", "()", self.size)
    }
}

impl DataStorageReadonly for ChunkedFile {
    type Block = Chunk4096;

    fn len(&self) -> Result<usize> {
        Ok((self.size / 4096) as usize + if self.size % 4096 == 0 { 0 } else { 1 })
    }

    fn is_empty(&self) -> Result<bool> {
        Ok(self.size == 0)
    }

    fn get(&self, index: usize) -> Result<Self::Block> {
        let offset = index as u64 * 4096;
        if offset >= self.size {
            Err(INDEX_IS_OUT_OF_BOUNDS)?;
        }
        let mut fd = self.fd.borrow_mut();
        fd.seek(SeekFrom::Start(offset))?;
        Ok(Chunk4096::from_stream(&mut *fd)?)
    }
}
