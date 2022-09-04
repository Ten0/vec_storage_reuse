# vec_storage_reuse

[![Crates.io](https://img.shields.io/crates/v/vec_storage_reuse.svg)](https://crates.io/crates/vec_storage_reuse)
[![License](https://img.shields.io/github/license/Ten0/vec_storage_reuse)](LICENSE)

Provides an API to reuse a `Vec`'s allocation.

This is useful to achieve zero-alloc when storing data with short lifetimes in a `Vec`:
```rust
    let mut objects_storage: VecStorageForReuse<Object<'static>> = VecStorageForReuse::new();

    while let Some(byte_chunk) = stream.next() { // byte_chunk only lives this scope
        let mut objects: &mut Vec<Object<'_>> = &mut *objects_storage.reuse_allocation();

        // Zero-copy parsing; Object has references to chunk
        deserialize(byte_chunk, &mut objects)?;
        process(&objects)?;
    } // byte_chunk lifetime ends
```

### Credits:
This crate delegates the actual unsafe functionality to the `recycle_vec` crate, and just provides
an interface that abstracts the swapping with the container through `Drop`, so that one can never
forget to swap back the temporary object with the storage
