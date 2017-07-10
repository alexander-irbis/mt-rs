use std::fmt;
use std::ops;

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
    /// Checks if a range is within bounds.
    /// Should be reused to check ranges by `Self::range()` implamentations
    fn check_range(&self, range: &ops::Range<usize>) -> Result<()> {
        let len = self.len()?;
        return if not_is_in(len, range) {
            Err(INDEX_IS_OUT_OF_BOUNDS)
        } else {
            Ok(())
        };

        #[cfg(not(feature="collections_range"))]
        fn not_is_in(len: usize, range: &ops::Range<usize>) -> bool {
            range.start >= len || range.end > len
        }

        #[cfg(feature="collections_range")]
        fn not_is_in(len: usize, range: &ops::Range<usize>) -> bool {
            use std::collections::Bound;
            use std::collections::range::RangeArgument;

            match range.start() {
                Bound::Excluded(x) if x >= len - 1 => true,
                Bound::Included(x) if x >= len => true,
                _ => false
            }
            ||
            match range.end() {
                Bound::Excluded(x) if x > len => true,
                Bound::Included(x) if x >= len => true,
                _ => false
            }
        }
    }

    /// Creates an iterator over range (or slice)
    fn range<'s: 'i, 'i>(&'s self, range: ops::Range<usize>) -> Result<Box<Iterator<Item=Result<Self::Block>> + 'i>> {
        self.check_range(&range)?;
        Ok(Box::new(range.map(move |index| self.get(index) )))
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
