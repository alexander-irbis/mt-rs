use std::fmt;

use prelude::*;


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

    pub fn get(&self, index: usize) -> Result<Option<<A::Algorithm as MTAlgorithm>::Value>> {
        self.tree.get_value(self.level, index)
    }
}


pub trait TreeStorage: fmt::Debug {
    type Algorithm: MTAlgorithm;

    /// Returns the number of levels in the tree
    fn len(&self) -> Result<usize>;

    /// Returns whether the tree is empty
    fn is_empty(&self) -> Result<bool> {
        self.len().map(|len| len == 0)
    }

    /// Clears all data
    fn clear(&mut self) -> Result<()> {
        self.clear_and_reserve(&[])
    }

    /// Clears all data and reserves space for levels
    fn clear_and_reserve(&mut self, sizes: &[usize]) -> Result<()>;

    /// Adds 1 level to the tree
    fn grow(&mut self) -> Result<()>;

    /// Returns an info about the specified level, if the level exists
    fn get_level(&self, level: usize) -> Result<Option<TreeLevel<Self>>> where Self: Sized {
        self.get_level_len(level).map(|len| len.map(|len| TreeLevel { len, level, tree: self }))
    }

    /// Returns a width of the specified level, if the level exists
    fn get_level_len(&self, level: usize) -> Result<Option<usize>>;

    /// Returns a value
    fn get_value(&self, level: usize, index: usize) -> Result<Option<<Self::Algorithm as MTAlgorithm>::Value>>;

    /// Returns a mutable reference
    fn get_value_mut(&mut self, level: usize, index: usize) -> Result<Option<&mut <Self::Algorithm as MTAlgorithm>::Value>>;

    /// Appends a value to the back of the specified level
    fn push(&mut self, level: usize, value: <Self::Algorithm as MTAlgorithm>::Value) -> Result<()>;

    fn extend<I>(&mut self, level: usize, other: I) -> Result<()>
        where I: IntoIterator<Item=<Self::Algorithm as MTAlgorithm>::Value>;

    fn extend_from_slice(&mut self, level: usize, slice: &[<Self::Algorithm as MTAlgorithm>::Value]) -> Result<()>;

    fn iter_level<'s>(&'s self, level: usize) -> Result<Option<Box<Iterator<Item=Result<<Self::Algorithm as MTAlgorithm>::Value>> + 's>>>;

    fn iter_level_by_pair<'s>(&'s self, level: usize) -> Result<Option<Box<Iterator<
        Item=Result<(<Self::Algorithm as MTAlgorithm>::Value, <Self::Algorithm as MTAlgorithm>::Value)>
    > + 's>>>;

    /// Returns root, if the tree is not empty
    fn get_root(&self) -> Result<Option<<Self::Algorithm as MTAlgorithm>::Value>> {
        if self.is_empty()? {
            return Ok(None);
        }
        let last_level = self.len()? - 1;
        let last_index = self.get_level_len(last_level)?.unwrap() - 1;
        self.get_value(last_level, last_index)
    }

    /// Returns root. Panics, if tree is empty.
    #[deprecated(since="0.1.0", note="please use `get_root` instead")]
    fn root(&self) -> Result<<Self::Algorithm as MTAlgorithm>::Value> {
        Ok(self.get_root()?.expect("Tree is empty"))
    }
}
