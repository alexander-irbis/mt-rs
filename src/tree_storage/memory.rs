use std::fmt;

use prelude::*;


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
        write!(f, "MemoryTreeStorage(len={})", self.len().map_err(|_| fmt::Error)?)
    }
}

impl <A> TreeStorage for MemoryTreeStorage<A> where A: MTAlgorithm {
    type Algorithm = A;

    fn len(&self) -> Result<usize> {
        Ok(self.layers.len())
    }

    fn is_empty(&self) -> Result<bool> {
        Ok(self.layers.is_empty())
    }

    fn clear_and_reserve(&mut self, sizes: &[usize]) -> Result<()> {
        self.layers.truncate(sizes.len());
        while self.layers.len() < sizes.len() {
            self.layers.push(Vec::new());
        }
        for (level, &size) in sizes.iter().enumerate() {
            let layer = &mut self.layers[level];
            layer.clear();
            layer.reserve(size);
        }
        Ok(())
    }

    fn grow(&mut self) -> Result<()> {
        self.layers.push(Vec::new());
        Ok(())
    }

    fn get_level_len(&self, level: usize) -> Result<usize> {
        self.layers.get(level)
            .map(Vec::len)
            .ok_or(INDEX_IS_OUT_OF_BOUNDS)
    }

    fn get_value(&self, level: usize, index: usize) -> Result<<Self::Algorithm as MTAlgorithm>::Value> {
        self.layers.get(level)
            .and_then(|layer| layer.get(index).cloned())
            .ok_or(INDEX_IS_OUT_OF_BOUNDS)
    }

    fn get_value_mut(&mut self, level: usize, index: usize) -> Result<&mut <Self::Algorithm as MTAlgorithm>::Value> {
        self.layers.get_mut(level)
            .and_then(|layer| layer.get_mut(index))
            .ok_or(INDEX_IS_OUT_OF_BOUNDS)
    }

    fn push(&mut self, level: usize, value: <Self::Algorithm as MTAlgorithm>::Value) -> Result<()> {
        let layer = self.layers.get_mut(level).ok_or(StateError::InconsistentState)?;
        layer.push(value);
        Ok(())
    }

    fn extend<I>(&mut self, level: usize, other: I) -> Result<()>
        where I: IntoIterator<Item=<Self::Algorithm as MTAlgorithm>::Value>
    {
        let layer = self.layers.get_mut(level).ok_or(StateError::InconsistentState)?;
        layer.extend(other);
        Ok(())
    }

    fn extend_from_slice(&mut self, level: usize, slice: &[<Self::Algorithm as MTAlgorithm>::Value]) -> Result<()> {
        let layer = self.layers.get_mut(level).ok_or(StateError::InconsistentState)?;
        layer.extend_from_slice(slice);
        Ok(())
    }

    fn iter_level<'s>(&'s self, level: usize) -> Result<Box<Iterator<Item=Result<<Self::Algorithm as MTAlgorithm>::Value>> + 's>> {
        self.layers.get(level)
            .map(|layer| Box::new(layer.iter().cloned().map(Ok)) as Box<Iterator<Item=_>>)
            .ok_or(INDEX_IS_OUT_OF_BOUNDS)
    }

    fn iter_level_by_pair<'s>(&'s self, level: usize) -> Result<Box<Iterator<
        Item=Result<(<Self::Algorithm as MTAlgorithm>::Value, <Self::Algorithm as MTAlgorithm>::Value)>
    > + 's>> {
        self.layers.get(level)
            .map(|layer| Box::new(layer.chunks(2).map(|chunk| {
                let i2 = (chunk.len() + 1) % 2;
                Ok((chunk[0].clone(), chunk[i2].clone()))
            })) as Box<Iterator<Item=_>>)
            .ok_or(INDEX_IS_OUT_OF_BOUNDS)
    }

    fn get_root(&self) -> Result<Option<<Self::Algorithm as MTAlgorithm>::Value>> {
        Ok(self.layers.iter().last().and_then(|layer| layer.iter().last().cloned()))
    }
}
