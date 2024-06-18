use std::{
    collections::{btree_map as map, BTreeMap},
    fmt::Debug,
    iter::FusedIterator,
    pin::Pin,
    sync::RwLockReadGuard,
};

/// Iterator over keys of [super::PinnedMap].
pub struct Keys<'a, K, V> {
    /// Shall not be read. Only kept here to prevent the map from being modified.
    #[allow(unused)]
    guard: RwLockReadGuard<'a, BTreeMap<K, Pin<Box<V>>>>,
    inner: map::Keys<'a, K, Pin<Box<V>>>,
}

impl<'a, K, V> Keys<'a, K, V> {
    pub(super) fn new(guard: RwLockReadGuard<'a, BTreeMap<K, Pin<Box<V>>>>) -> Self {
        let inner = unsafe { std::mem::transmute(guard.keys()) };
        Self { guard, inner }
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<&'a K> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn last(mut self) -> Option<&'a K> {
        self.next_back()
    }

    fn min(mut self) -> Option<&'a K>
    where
        &'a K: Ord,
    {
        self.next()
    }

    fn max(mut self) -> Option<&'a K>
    where
        &'a K: Ord,
    {
        self.next_back()
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        self.inner.next_back()
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Keys<'_, K, V> {}

impl<K: Debug, V> Debug for Keys<'_, K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}
