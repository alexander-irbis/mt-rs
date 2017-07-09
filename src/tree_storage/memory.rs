use std::fmt;

use abc::*;


pub struct MemoryTreeStorage<A> where A: MTAlgorithm {
    // Hashes are stored as layers
    // In the begin (index 0) is the bottom level 0 with hashes of the data
    // next layers keep hashes of previous levels, till the root
    layers: Vec<Vec<A::Value>>
}

impl <A> Default for MemoryTreeStorage<A> where A: MTAlgorithm {
    fn default() -> Self {
        MemoryTreeStorage {
            layers: Vec::new()
        }
    }
}

impl <A> MemoryTreeStorage<A> where A: MTAlgorithm {
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut Vec<Vec<A::Value>> {
        &mut self.layers
    }
}

impl <A> fmt::Debug for MemoryTreeStorage<A> where A: MTAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryTreeStorage(len={})", self.len())
    }
}

impl <A> TreeStorage for MemoryTreeStorage<A> where A: MTAlgorithm {
    type Algorithm = A;
    type StorageError = ();

    fn len(&self) -> usize {
        self.layers.len()
    }

    fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    fn clear_and_reserve(&mut self, sizes: &[usize]) {
        self.layers.truncate(sizes.len());
        while self.layers.len() < sizes.len() {
            self.layers.push(Vec::new());
        }
        for (level, &size) in sizes.iter().enumerate() {
            let layer = &mut self.layers[level];
            layer.clear();
            layer.reserve(size);
        }
    }

    fn grow(&mut self) {
        self.layers.push(Vec::new())
    }

    fn get_level_len(&self, level: usize) -> Option<usize> {
        self.layers.get(level).map(Vec::len)
    }

    fn get_value(&self, level: usize, index: usize) -> Option<&<Self::Algorithm as MTAlgorithm>::Value> {
        self.layers.get(level).and_then(|layer| layer.get(index))
    }

    fn get_value_mut(&mut self, level: usize, index: usize) -> Option<&mut <Self::Algorithm as MTAlgorithm>::Value> {
        self.layers.get_mut(level).and_then(|layer| layer.get_mut(index))
    }

    fn push(&mut self, level: usize, value: <Self::Algorithm as MTAlgorithm>::Value) -> Result<(), Self::StorageError> {
        let layer = self.layers.get_mut(level).ok_or(())?;
        layer.push(value);
        Ok(())
    }

    fn extend<I>(&mut self, level: usize, other: I) -> Result<(), Self::StorageError>
        where I: IntoIterator<Item=<Self::Algorithm as MTAlgorithm>::Value>
    {
        let layer = self.layers.get_mut(level).ok_or(())?;
        layer.extend(other);
        Ok(())
    }

    fn extend_from_slice(&mut self, level: usize, slice: &[<Self::Algorithm as MTAlgorithm>::Value]) -> Result<(), Self::StorageError> {
        let layer = self.layers.get_mut(level).ok_or(())?;
        layer.extend_from_slice(slice);
        Ok(())
    }

    fn iter_level<'s>(&'s self, level: usize) -> Option<Box<Iterator<Item=&'s <Self::Algorithm as MTAlgorithm>::Value> + 's>> {
        self.layers.get(level).map(|layer| Box::new(layer.iter()) as Box<Iterator<Item=_>>)
    }

    fn iter_level_by_pair<'s>(&'s self, level: usize) -> Option<Box<Iterator<
        Item=(&'s <Self::Algorithm as MTAlgorithm>::Value, &'s <Self::Algorithm as MTAlgorithm>::Value)
    > + 's>> {
        self.layers.get(level).map(|layer| Box::new(layer.chunks(2).map(|chunk| {
            let i2 = (chunk.len() + 1) % 2;
            (&chunk[0], &chunk[i2])
        })) as Box<Iterator<Item=_>>)
    }

    fn try_root(&self) -> Option<&<Self::Algorithm as MTAlgorithm>::Value> {
        self.layers.iter().last().and_then(|layer| layer.iter().last())
    }
}
