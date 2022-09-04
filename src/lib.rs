//! Provides an API to reuse a `Vec`'s allocation.
//!
//! This is useful to achieve zero-alloc when storing data with short lifetimes in a `Vec`:
//! ```
//! # use std::error::Error;
//! # use vec_storage_reuse::VecStorageForReuse;
//! #
//! # struct Stream;
//! #
//! # impl Stream {
//! #     fn new() -> Self {
//! #         Stream
//! #     }
//! #
//! #     fn next(&mut self) -> Option<&[u8]> {
//! #         Some(&b"hoge"[..])
//! #     }
//! # }
//! #
//! # fn process(input: &[Object<'_>]) -> Result<(), Box<dyn Error>> {
//! #     Ok(())
//! # }
//! #
//! # struct Object<'a> {
//! #     reference: &'a [u8],
//! # }
//! #
//! # fn deserialize<'a>(input: &'a [u8], output: &mut Vec<Object<'a>>) -> Result<(), Box<dyn Error>> {
//! #     output.push(Object { reference: input });
//! #     Ok(())
//! # }
//! #
//! # fn processor() -> Result<(), Box<dyn Error>> {
//! #    let mut stream = Stream::new();
//! #    
//!     let mut objects_storage: VecStorageForReuse<Object<'static>> = VecStorageForReuse::new();
//!
//!     while let Some(byte_chunk) = stream.next() { // byte_chunk only lives this scope
//!         let mut objects: &mut Vec<Object<'_>> = &mut *objects_storage.reuse_allocation();
//!
//!         // Zero-copy parsing; Object has references to chunk
//!         deserialize(byte_chunk, &mut objects)?;
//!         process(&objects)?;
//!     } // byte_chunk lifetime ends
//! #
//! #    Ok(())
//! # }
//! ```
//!
//! ### Credits:
//! This crate delegates the actual unsafe functionality to the `recycle_vec` crate, and just provides
//! an interface that abstracts the swapping with the container through `Drop`, so that one can never
//! forget to swap back the temporary object with the storage

extern crate recycle_vec;

use std::ops::{Deref, DerefMut, Drop};

/// Implements `DerefMut<Target = Vec<T>>`, and puts the allocation back in place
/// in the source `Vec<S>` once dropped
pub struct VecStorageReuse<'a, T, S> {
    storage: &'a mut Vec<S>,
    inner: Vec<T>,
}

impl<'a, T, S> VecStorageReuse<'a, T, S> {
    /// Allows re-interpreting the type of a Vec to reuse the allocation.
    /// The vector is emptied and any values contained in it will be dropped.
    /// The target type must have the same size and alignment as the source type.
    ///
    /// # Panics
    /// Panics if the size or alignment of the source and target types don't match.
    pub fn new(storage: &'a mut Vec<S>) -> Self {
        Self {
            inner: recycle_vec::VecExt::recycle(std::mem::replace(storage, Vec::new())),
            storage,
        }
    }
}

impl<'a, T, S> Drop for VecStorageReuse<'a, T, S> {
    fn drop(&mut self) {
        *self.storage =
            recycle_vec::VecExt::recycle(std::mem::replace(&mut self.inner, Vec::new()));
    }
}

impl<T, S> Deref for VecStorageReuse<'_, T, S> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T, S> DerefMut for VecStorageReuse<'_, T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Stores a Vec and prevents it from being accessed in any other ways than through reinterpreting its type
/// to reuse the allocation.
/// This is useful to make it clear by typing that it's its only intended purpose.
pub struct VecStorageForReuse<S> {
    inner: Vec<S>,
}

impl<S> VecStorageForReuse<S> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Uses the inner `Vec<S>` storage to provide a `VecStorageReuse: DerefMut<Target = Vec<T>>`
    ///
    /// This avoids reallocating a new `Vec<T>`.
    pub fn reuse_allocation<'a, T>(&'a mut self) -> VecStorageReuse<'a, T, S> {
        VecStorageReuse::new(&mut self.inner)
    }

    pub fn from_vec(vec_to_use_as_storage: Vec<S>) -> Self {
        Self {
            inner: vec_to_use_as_storage,
        }
    }

    pub fn into_inner(self) -> Vec<S> {
        self.inner
    }
}
