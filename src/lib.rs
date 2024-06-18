//! Mutable containers for pinned and immutable items.
//!
//! A substitute for [Box::leak](https://doc.rust-lang.org/stable/alloc/boxed/struct.Box.html#method.leak).

#![warn(missing_docs, rust_2021_compatibility, rust_2018_idioms)]

extern crate alloc;

const PANIC: &'static str = "Another thread panicked while holding the lock.";

mod list;
mod map;

pub use list::PinnedList;
pub use map::{Iter, Keys, PinnedMap, Values};
