## Merkle Tree 

### Unstable

> A prototype implementation. Use at your own risk.


## Example

```rust
extern crate mt;

use mt::data_storage::MemoryDataStorage;
use mt::tree_storage::MemoryTreeStorage;
use mt::merkle_tree::MerkleTree;
use mt::fun::Sha256;
use mt::fun::DoubleHash;

// ....

let mut mt: MerkleTree<MemoryDataStorage<&[u8]>, MemoryTreeStorage<DoubleHash<Sha256>>>;
mt = MerkleTree::default();

// ....

mt.push(&b"123")?;
mt.push(&b"456")?;
mt.push(&b"7890")?;

// ....

mt.check_data()?;
mt.check_tree()?;

// ....

mt.get_root()? == other_root
```


## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
