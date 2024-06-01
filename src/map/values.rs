use std::{
    collections::{btree_map as map, BTreeMap},
    fmt::Debug,
    iter::FusedIterator,
    pin::Pin,
    sync::RwLockReadGuard,
};

use super::erase;

pub struct Values<'a, K, V> {
    /// Shall not be read. Only kept here to prevent the map from being modified.
    #[allow(unused)]
    guard: RwLockReadGuard<'a, BTreeMap<K, Pin<Box<V>>>>,
    inner: map::Values<'a, K, Pin<Box<V>>>,
}

impl<'a, K, V> Values<'a, K, V> {
    pub fn new(guard: RwLockReadGuard<'a, BTreeMap<K, Pin<Box<V>>>>) -> Self {
        let inner = unsafe { std::mem::transmute(guard.values()) };
        Self { guard, inner }
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<&'a V> {
        self.inner.next().map(erase)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn last(mut self) -> Option<&'a V> {
        self.next_back()
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.inner.next_back().map(erase)
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> {}

impl<K, V: Debug> Debug for Values<'_, K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}
