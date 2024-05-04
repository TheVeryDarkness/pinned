use super::PANIC;
use alloc::boxed::Box;
use core::{iter::Map, mem, ops::Deref, pin::Pin};
use std::{
    collections::{
        btree_map::{Keys, Values},
        BTreeMap,
    },
    sync::RwLock,
};

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
///
/// The same to [PinnedMap::keys].
///
/// ```compile_fail
/// use pinned_bucket::*;
/// let v = PinnedMap::new();
/// v.insert(1, 2);
/// let keys = v.keys();
/// drop(v);
/// assert_eq!(format!("{:?}", keys), "[1]");
/// ```
///
/// If you [clone](Clone::clone) this,
/// references to items in new container will be different to
/// references to those in old container.
#[derive(Debug)]
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
    /// Get an item in [PinnedMap] if there exists one,
    /// otherwise push an item into the [PinnedMap]
    /// and return the reference to it.
    pub fn get_or_insert(&self, key: K, value: V) -> &V
    where
        K: Ord,
    {
        let mut guard = self.sections.write().expect(PANIC);
        let v = guard.entry(key).or_insert(Box::pin(value));
        let r = v.deref();
        unsafe { mem::transmute::<&V, &V>(r) }
    }
    /// Get an item in [PinnedMap] if there exists one,
    /// otherwise push an item into the [PinnedMap]
    /// and return the reference to it.
    pub fn get_or_insert_with(&self, key: K, default: impl FnOnce() -> V) -> &V
    where
        K: Ord,
    {
        let mut guard = self.sections.write().expect(PANIC);
        let v = guard.entry(key).or_insert_with(|| Box::pin(default()));
        let r = v.deref();
        unsafe { mem::transmute::<&V, &V>(r) }
    }
    /// Get all keys.
    pub fn keys(&self) -> Keys<'_, K, Pin<Box<V>>>
    where
        K: Ord,
    {
        let guard = self.sections.read().expect(PANIC);
        unsafe { mem::transmute(guard.keys()) }
    }
    /// Get all values.
    pub fn values(&self) -> Map<Values<'_, K, Pin<Box<V>>>, fn(&Pin<Box<V>>) -> &V>
    where
        K: Ord,
    {
        let guard = self.sections.read().expect(PANIC);
        let values = guard.values();
        let values: Values<'_, K, Pin<Box<V>>> = unsafe { mem::transmute(values) };
        values.map(|value| unsafe { mem::transmute(value.as_ref().deref()) })
    }
}
impl<K: Clone, V: Clone> Clone for PinnedMap<K, V> {
    fn clone(&self) -> Self {
        let values = self.sections.read().expect(PANIC);
        let sections = values.clone().into();
        Self { sections }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unreachable<T>() -> T {
        unreachable!()
    }

    #[test]
    fn it_works() {
        let v = PinnedMap::new();
        let a = v.insert(1, 2);
        let b = v.insert(2, 3);
        let a_ = v.get_or_insert(1, -1);
        let b_ = v.get_or_insert_with(2, unreachable);

        assert_eq!(v.len(), 2);

        let c = v.get_or_insert(3, 4);

        assert_eq!(v.len(), 3);

        let d = v.get_or_insert_with(4, || 5);

        v.get_or_insert_with(4, unreachable);

        assert_eq!(v.len(), 4);

        assert_eq!(a, &2);
        assert_eq!(a, v.get(&1).unwrap());
        assert_eq!(a as *const i32, v.get(&1).unwrap() as *const i32);
        assert_eq!(a, a_);

        assert_eq!(b, &3);
        assert_eq!(b, v.get(&2).unwrap());
        assert_eq!(b as *const i32, v.get(&2).unwrap() as *const i32);
        assert_eq!(b, b_);

        assert_eq!(c, &4);
        assert_eq!(c, v.get(&3).unwrap());
        assert_eq!(c as *const i32, v.get(&3).unwrap() as *const i32);

        assert_eq!(d, &5);
        assert_eq!(d, v.get(&4).unwrap());
        assert_eq!(d as *const i32, v.get(&4).unwrap() as *const i32);

        assert_eq!(v.values().cloned().collect::<Vec<_>>(), vec![2, 3, 4, 5]);
    }

    #[test]
    fn debug_list() {
        let v: PinnedMap<usize, usize> = PinnedMap::default();
        v.insert(1, 2);
        v.insert(3, 4);
        let u = v.clone();
        assert_eq!(format!("{:?}", v), format!("{:?}", u));
        assert_eq!(v.values().cloned().collect::<Vec<_>>(), vec![2, 4]);
    }

    #[test]
    fn debug_keys() {
        let v: PinnedMap<usize, String> = PinnedMap::default();
        v.insert(1, "1".into());
        v.insert(2, "2".into());
        assert_eq!(format!("{:?}", v.keys()), "[1, 2]");
        assert_eq!(
            format!("{:?}", v.sections.read().unwrap()),
            "{1: \"1\", 2: \"2\"}",
        );
        assert_eq!(v.values().collect::<Vec<_>>(), vec!["1", "2"]);
    }

    #[test]
    fn insert_with() {
        let v = PinnedMap::new();
        v.insert(1, "1".to_owned());
        v.insert(2, "2".to_owned());
        v.get_or_insert_with(2, unreachable);
    }

    #[test]
    #[should_panic]
    fn insert_with_panicked() {
        let v = PinnedMap::new();
        v.insert(1, "1".to_owned());
        v.insert(2, "2".to_owned());
        v.get_or_insert_with(2, unreachable);
        v.get_or_insert_with(3, unreachable);
    }
}
