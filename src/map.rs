use super::PANIC;
use alloc::boxed::Box;
use core::{mem, ops::Deref, pin::Pin};
use std::{collections::BTreeMap, sync::RwLock};

/// A map from `K` to `Pin<Box<V>>`.
///
/// One can keep the references to a lot of pinned items,
/// whose lifetime is managed by the container,
/// without holding a mutable reference to the container.
///
/// ```rust
/// use pinned_bucket::*;
/// let v = PinnedMap::new();
/// let a = v.insert(1, 2);
/// let b = v.insert(3, 4);
/// assert_eq!(a, &2);
/// assert_eq!(b, &4);
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
/// let v = PinnedMap::new();
/// let a = v.insert(1, 2);
/// drop(v);
/// assert_eq!(a, &1);
/// ```
pub struct PinnedMap<K, V> {
    sections: RwLock<BTreeMap<K, Pin<Box<V>>>>,
}
impl<K, V> Default for PinnedMap<K, V> {
    fn default() -> Self {
        Self {
            sections: RwLock::new(BTreeMap::new()),
        }
    }
}
impl<K, V> PinnedMap<K, V> {
    /// Create an empty [PinnedMap].
    pub fn new() -> Self {
        Self::default()
    }
    /// Get the number of elements in [PinnedMap].
    pub fn len(&self) -> usize {
        self.sections.read().expect(PANIC).len()
    }
    /// Push an item into the [PinnedMap]
    /// and return the reference to it.
    pub fn insert(&self, key: K, value: V) -> &V
    where
        K: Ord,
    {
        let item = Box::pin(value);
        let r = item.deref();
        let r: &V = unsafe { mem::transmute::<&V, &V>(r) };
        self.sections.write().expect(PANIC).insert(key, item);
        r
    }
    /// Get an item in [PinnedMap].
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        self.sections.read().expect(PANIC).get(key).map(|v| {
            let r = v.deref();
            unsafe { mem::transmute::<&V, &V>(r) }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let v = PinnedMap::new();
        let a = v.insert(1, 2);
        let b = v.insert(2, 3);

        assert_eq!(v.len(), 2);

        assert_eq!(a, &2);
        assert_eq!(a, v.get(&1).unwrap());
        assert_eq!(a as *const i32, v.get(&1).unwrap() as *const i32);

        assert_eq!(b, &3);
        assert_eq!(b, v.get(&2).unwrap());
        assert_eq!(b as *const i32, v.get(&2).unwrap() as *const i32);
    }
}
