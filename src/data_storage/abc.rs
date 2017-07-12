use std::fmt;
use std::ops;

use prelude::*;


/// Static data (for example, on read-only media).
/// Can only be checked
/// Any tree storage backend should implement this trait
pub trait DataStorageReadonly: fmt::Debug {
    type DataValue: MTHash;

    /// Returns the length of the collection
    fn len(&self) -> Result<usize>;

    /// Returns true if collection is empty
    fn is_empty(&self) -> Result<bool> {
        self.len().map(|len| len == 0)
    }

    /// Returns an item at index, or error, if index out of bounds
    fn get(&self, index: usize) -> Result<Self::DataValue>;

    /// Return an iterator over all elements in collection
    fn iter<'s: 'i, 'i>(&'s self) -> Result<Box<Iterator<Item=Result<Self::DataValue>> + 'i>> {
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
    fn range<'s: 'i, 'i, R: Into<ops::Range<usize>>>(&'s self, range: R) -> Result<Box<Iterator<Item=Result<Self::DataValue>> + 'i>> {
        let range = range.into();
        self.check_range(&range)?;
        Ok(Box::new(range.map(move |index| self.get(index) )))
    }

    /// Writable data storage have to override this method
    fn is_writeable(&self) -> bool {
        false
    }
}


/// Any writable tree storage backend should implement this trait
pub trait DataStorage: DataStorageReadonly {
    /// Appends an item to the back of the collection
    fn push(&mut self, data: Self::DataValue) -> Result<()>;

    /// Appends items to the back of the collection
    fn extend<DD: IntoIterator<Item=Result<Self::DataValue>>>(&mut self, data: DD) -> Result<()>;

    /// Clears all data
    fn clear(&mut self) -> Result<()>;
}
