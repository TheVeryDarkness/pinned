use super::PANIC;
use alloc::{boxed::Box, vec::Vec};
use core::{
    mem,
    ops::{Deref, Index},
    pin::Pin,
};
use std::sync::RwLock;

/// A list of `Pin<Box<T>>`.
///
/// One can keep the references to a lot of pinned items,
/// whose lifetime is managed by the container,
/// without holding a mutable reference to the container.
///
/// ```rust
/// use pinned_bucket::*;
/// let v = PinnedList::new();
/// let a = v.push(1);
/// let b = v.push(2);
/// assert_eq!(a, &1);
/// assert_eq!(b, &2);
/// ```
///
/// By the way, I cannot implement this without a lock inside.
/// Borrow checker isn't so smart currently.
///
/// As the items inside are still managed by the container,
/// codes below won't compile.
///
/// ```compile_fail
/// use pinned_bucket::*;
/// let v = PinnedList::new();
/// let a = v.push(1);
/// drop(v);
/// assert_eq!(a, &1);
/// ```
///
/// If you [clone](Clone::clone) this,
/// references to items in new container will be different to
/// references to those in old container.
#[derive(Debug)]
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
    /// Create a [PinnedList] with given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sections: Vec::with_capacity(capacity).into(),
        }
    }
    /// Get current capacity.
    pub fn capacity(&self) -> usize {
        self.sections.read().expect(PANIC).capacity()
    }
    /// Get the number of elements in [PinnedList].
    pub fn len(&self) -> usize {
        self.sections.read().expect(PANIC).len()
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
impl<T: Clone> Clone for PinnedList<T> {
    fn clone(&self) -> Self {
        let values = self.sections.read().expect(PANIC);
        let sections = values.clone().into();
        Self { sections }
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
        assert_eq!(v.len(), 2);
    }

    /// To ensure that allocated items won't be moved.
    #[test]
    fn resize() {
        let v = PinnedList::with_capacity(4);
        let cap = v.capacity();
        let refs: Vec<_> = (0..cap + 1)
            .into_iter()
            .map(|i| {
                let r = v.push(i);
                (r, r as *const usize)
            })
            .collect();
        assert_eq!(v.len(), cap + 1);
        let first = &v[0];
        let first = first as *const usize;
        assert_eq!(first, refs[0].1);
    }

    #[test]
    fn extend_resize() {
        let v: PinnedList<usize> = PinnedList::with_capacity(4);
        let former: Vec<_> = v.extend((0..4).into_iter());
        let _latter: Vec<_> = v.extend((0..4).into_iter());
        for i in 0..4 {
            assert_eq!(former[i], &v[i]);
            assert_eq!(former[i] as *const usize, &v[i] as *const usize);
        }
        assert_eq!(v.len(), 4 + 4);
    }

    #[test]
    fn debug_list() {
        let v: PinnedList<usize> = PinnedList::with_capacity(2);
        let _: Vec<_> = v.extend((0..4).into_iter());
        let u = v.clone();
        assert_eq!(format!("{:?}", v), format!("{:?}", u));
    }
}
