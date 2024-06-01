use std::{
    collections::{btree_map as map, BTreeMap},
    fmt::Debug,
    iter::FusedIterator,
    pin::Pin,
    sync::RwLockReadGuard,
};

use super::erase;

pub struct Iter<'a, K, V> {
    /// Shall not be read. Only kept here to prevent the map from being modified.
    #[allow(unused)]
    guard: RwLockReadGuard<'a, BTreeMap<K, Pin<Box<V>>>>,
    inner: map::Iter<'a, K, Pin<Box<V>>>,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub fn new(guard: RwLockReadGuard<'a, BTreeMap<K, Pin<Box<V>>>>) -> Self {
        let inner = unsafe { std::mem::transmute(guard.values()) };
        Self { guard, inner }
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.inner.next().map(|(k, v)| (k, erase(v)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn last(self) -> Option<(&'a K, &'a V)> {
        self.inner.last().map(|(k, v)| (k, erase(v)))
    }

    fn min(mut self) -> Option<(&'a K, &'a V)>
    where
        (&'a K, &'a V): Ord,
    {
        self.next()
    }

    fn max(mut self) -> Option<(&'a K, &'a V)>
    where
        (&'a K, &'a V): Ord,
    {
        self.next_back()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        self.inner.next_back().map(|(k, v)| (k, erase(v)))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K: Debug, V: Debug> Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}
