use abc::*;

#[derive(Debug, Default)]
pub struct MerkleTree<H> where H: MTHashFunction {
    // Hashes are stored as layers
    // In the begin (index 0) is the bottom level 0 with hashes of the data
    // next layers keep hashes of previous levels, till the root
    layers: Vec<Vec<H::Value>>,
}

impl <H> MerkleTree<H> where H: MTHashFunction {
    pub fn new() -> Self {
        MerkleTree {
            layers: Vec::new(),
        }
    }

    pub fn push_with_data<D: MTHashable>(&mut self, data: &D) {
        let hash = H::eval_hash(data);
        self.push_hash(0, hash)
    }

    fn push_hash(&mut self, layer: usize, hash: H::Value) {
        debug_assert!(layer <= self.layers.len());
        if self.layers.len() == layer {
            self.layers.push(Vec::new());
        }
        self.layers[layer].push(hash);
        self.update_branch(layer)
    }

    fn update_branch(&mut self, layer: usize) {
        let layer_is_last = self.layers.len() == layer + 1;
        let len = self.layers[layer].len();
        if len > 1 {
            let hash = {
                let layer = &self.layers[layer];
                let pair = if len % 2 == 0 {
                    (&layer[len - 2], &layer[len - 1])
                } else {
                    (&layer[len - 1], &layer[len - 1])
                };
                H::eval_hash(&pair)
            };
            if layer_is_last || len % 2 == 1 {
                self.push_hash(layer + 1, hash);
            } else {
                {
                    let top_layer = &mut self.layers[layer + 1];
                    let top_len = top_layer.len();
                    debug_assert!(top_len > 0);
                    top_layer[top_len - 1] = hash;
                }
                self.update_branch(layer + 1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use abc::*;
    use super::MerkleTree;
    use fun::defaulthash::DefaultHash;

    #[test]
    fn merkle_tree_works() {
        let data: [&[u8]; 3] = [b"123", b"321", b"555"];
        let mut tree = MerkleTree::<DefaultHash>::new();
        tree.push_with_data(&data[0]);
        tree.push_with_data(&data[1]);
        tree.push_with_data(&data[2]);
        println!("{:?}", tree)
    }
}
