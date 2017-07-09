use std::fmt;

use fun::abc::*;


pub struct TreeLevel<'a, A: TreeStorage + 'a> {
    len: usize,
    level: usize,
    tree: &'a A,
}

impl<'a, A: TreeStorage + 'a> TreeLevel<'a, A> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, index: usize) -> Option<&<A::Algorithm as MTAlgorithm>::Value> {
        self.tree.get_value(self.level, index)
    }
}


pub trait TreeStorage: fmt::Debug {
    type Algorithm: MTAlgorithm;
    type StorageError: fmt::Debug;

    /// Returns the number of levels in the tree
    fn len(&self) -> usize;

    /// Returns whether the tree is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all data and reserves space for levels
    fn clear_and_reserve(&mut self, sizes: &[usize]);

    /// Adds 1 level to the tree
    fn grow(&mut self);

    /// Returns an info about the specified level, if the level exists
    fn get_level(&self, level: usize) -> Option<TreeLevel<Self>> where Self: Sized {
        self.get_level_len(level).map(|len| TreeLevel { len, level, tree: self })
    }

    /// Returns a width of the specified level, if the level exists
    fn get_level_len(&self, level: usize) -> Option<usize>;

    /// Returns a value
    fn get_value(&self, level: usize, index: usize) -> Option<&<Self::Algorithm as MTAlgorithm>::Value>;

    /// Returns a mutable reference
    fn get_value_mut(&mut self, level: usize, index: usize) -> Option<&mut <Self::Algorithm as MTAlgorithm>::Value>;

    /// Appends a value to the back of the specified level
    fn push(&mut self, level: usize, value: <Self::Algorithm as MTAlgorithm>::Value) -> Result<(), Self::StorageError>;

    fn extend<I>(&mut self, level: usize, other: I) -> Result<(), Self::StorageError>
        where I: IntoIterator<Item=<Self::Algorithm as MTAlgorithm>::Value>;

    fn extend_from_slice(&mut self, level: usize, slice: &[<Self::Algorithm as MTAlgorithm>::Value]) -> Result<(), Self::StorageError>;

    fn iter_level<'s>(&'s self, level: usize) -> Option<Box<Iterator<Item=&'s <Self::Algorithm as MTAlgorithm>::Value> + 's>>;

    // TODO stub with .iter_level
    fn iter_level_by_pair<'s>(&'s self, level: usize) -> Option<Box<Iterator<
        Item=(&'s <Self::Algorithm as MTAlgorithm>::Value, &'s <Self::Algorithm as MTAlgorithm>::Value)
    > + 's>>;

    fn try_root(&self) -> Option<&<Self::Algorithm as MTAlgorithm>::Value> {
        if self.is_empty() {
            return None;
        }
        let last_level = self.len() - 1;
        let last_index = self.get_level_len(last_level).unwrap() - 1;
        self.get_value(last_level, last_index)
    }

    fn root(&self) -> &<Self::Algorithm as MTAlgorithm>::Value {
        self.try_root().expect("Tree is empty")
    }
}
