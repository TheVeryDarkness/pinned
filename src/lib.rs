//! A mutable container for pinned and immutable items.
//!
//! A substitute for [Box::leak](https://doc.rust-lang.org/stable/alloc/boxed/struct.Box.html#method.leak).

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::{
    mem,
    ops::{Deref, Index},
    pin::Pin,
};
use std::sync::RwLock;

const PANIC: &'static str = "Another thread panicked while holding the lock.";

/// A list of `Pin<Box<T>>`.
///
/// I cannot implement this without a lock inside.
///
/// Borrow checker isn't so smart currently.
///
/// ```compile_fail
/// use pinned::PinnedVec;
/// let v = PinnedVec::new();
/// let a = v.push(1);
/// drop(v);
/// assert_eq!(a, &1);
/// ```
pub struct PinnedList<T> {
    sections: RwLock<Vec<Pin<Box<T>>>>,
}
impl<T> Default for PinnedList<T> {
    fn default() -> Self {
        Self {
            sections: RwLock::new(Vec::new()),
        }
    }
}
impl<T> PinnedList<T> {
    /// Create an empty [PinnedList].
    pub fn new() -> Self {
        Self::default()
    }
    ///
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sections: Vec::with_capacity(capacity).into(),
        }
    }
    /// Push an item into the [PinnedList]
    /// and return the reference to it.
    pub fn push(&self, t: T) -> &T {
        let item = Box::pin(t);
        let r = item.deref();
        let r: &T = unsafe { mem::transmute::<&T, &T>(r) };
        self.sections.write().expect(PANIC).push(item);
        r
    }
    /// Push a lot of items into the [PinnedList].
    pub fn extend<'s, U: IntoIterator<Item = T>, V: FromIterator<&'s T>>(&'s self, iter: U) -> V {
        let mut sec = self.sections.write().expect(PANIC);
        let len = sec.len();
        sec.extend(iter.into_iter().map(|item| Box::pin(item)));
        sec[len..]
            .iter()
            .map(|item| {
                let r = item.deref();
                let r: &'s T = unsafe { mem::transmute::<&T, &T>(r) };
                r
            })
            .collect()
    }
}
impl<T, I> Index<I> for PinnedList<T>
where
    Vec<Pin<Box<T>>>: Index<I, Output = Pin<Box<T>>>,
{
    type Output = T;
    fn index(&self, index: I) -> &Self::Output {
        let sec = self.sections.read().expect(PANIC);
        let r = sec.index(index).deref();
        unsafe { mem::transmute::<&T, &T>(r) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let v = PinnedList::new();
        let a = v.push(1);
        let b = v.push(2);
        assert_eq!(a, &1);
        assert_eq!(b, &2);
    }
}
