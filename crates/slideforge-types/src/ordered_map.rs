//! An order-preserving map that implements `Hash`.
//!
//! [`OrderedMap`] wraps [`indexmap::IndexMap`] and provides a `Hash`
//! implementation by hashing each key-value pair in insertion order.
//! This makes it suitable for use in IR types that must implement `Hash`
//! (for comemo compatibility and Kani bounded-model checking).
//!
//! ## Hash stability
//!
//! The hash value of an `OrderedMap` depends on both the set of entries AND
//! their insertion order. Two maps with the same key-value pairs in different
//! orders will hash differently — this is intentional for slideforge because
//! field and variable ordering is semantically significant (merge semantics
//! use last-wins on identical keys, and output ordering tracks source order).

use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use indexmap::IndexMap;

/// An insertion-order-preserving map that implements `Hash`, `Eq`, and `Clone`.
///
/// Hashing is stable only when both the contents and the insertion order are
/// equal. See the module documentation for rationale.
///
/// Implements `Deref<Target = IndexMap<K, V>>` so all `IndexMap` methods are
/// available without wrapping every call.
#[derive(Debug, Clone)]
pub struct OrderedMap<K: Hash + Eq, V>(pub IndexMap<K, V>);

impl<K: Hash + Eq + PartialEq, V: PartialEq> PartialEq for OrderedMap<K, V> {
    /// Order-sensitive equality: two maps are equal only if they have the same
    /// key-value pairs in the same insertion order. This is consistent with
    /// the order-sensitive `Hash` impl and reflects that field ordering is
    /// semantically significant in slideforge (merge semantics, PPTX output order).
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(other.0.iter())
                .all(|((k1, v1), (k2, v2))| k1 == k2 && v1 == v2)
    }
}

impl<K: Hash + Eq, V: Eq> Eq for OrderedMap<K, V> {}

impl<K: Hash + Eq, V: Hash> Hash for OrderedMap<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the entry count first to distinguish empty from non-empty.
        self.0.len().hash(state);
        // Hash each key-value pair in insertion order.
        for (k, v) in &self.0 {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl<K: Hash + Eq, V> Deref for OrderedMap<K, V> {
    type Target = IndexMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: Hash + Eq, V> DerefMut for OrderedMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K: Hash + Eq, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        OrderedMap(IndexMap::new())
    }
}

impl<K: Hash + Eq, V> OrderedMap<K, V> {
    /// Construct a new empty `OrderedMap`.
    #[must_use]
    pub fn new() -> Self {
        OrderedMap(IndexMap::new())
    }

    /// Insert a key-value pair. Semantics are identical to [`IndexMap::insert`].
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }
}

impl<K: Hash + Eq, V> IntoIterator for OrderedMap<K, V> {
    type Item = (K, V);
    type IntoIter = indexmap::map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K: Hash + Eq, V> IntoIterator for &'a OrderedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = indexmap::map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<K: Hash + Eq, V> FromIterator<(K, V)> for OrderedMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        OrderedMap(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_bc_1_01_ordered_map_hash_stable() {
        let mut m1: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m1.insert(Arc::from("a"), 1);
        m1.insert(Arc::from("b"), 2);

        let mut m2: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m2.insert(Arc::from("a"), 1);
        m2.insert(Arc::from("b"), 2);

        // Same content and order → same hash.
        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        m1.hash(&mut h1);
        m2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_bc_1_01_ordered_map_hash_order_matters() {
        let mut m1: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m1.insert(Arc::from("a"), 1);
        m1.insert(Arc::from("b"), 2);

        let mut m2: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m2.insert(Arc::from("b"), 2);
        m2.insert(Arc::from("a"), 1);

        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        m1.hash(&mut h1);
        m2.hash(&mut h2);
        // Different order → different hash (order-sensitive hashing).
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_bc_1_01_ordered_map_usable_as_hashmap_key() {
        let mut m: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m.insert(Arc::from("x"), 42);
        let mut outer: HashMap<OrderedMap<Arc<str>, i32>, &str> = HashMap::new();
        outer.insert(m, "my map");
        assert_eq!(outer.len(), 1);
    }

    #[test]
    fn test_bc_1_01_ordered_map_deref_gives_indexmap_api() {
        let mut m: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m.insert(Arc::from("key"), 99);
        // Deref to IndexMap to use its API directly.
        assert_eq!(m.get("key"), Some(&99));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn test_bc_1_01_ordered_map_clone() {
        let mut m: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m.insert(Arc::from("a"), 1);
        let m2 = m.clone();
        assert_eq!(m, m2);
    }

    #[test]
    fn test_bc_1_01_ordered_map_default_is_empty() {
        let m: OrderedMap<Arc<str>, i32> = OrderedMap::default();
        assert!(m.is_empty());
    }

    #[test]
    fn test_ordered_map_eq_is_order_sensitive() {
        let mut m1: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m1.insert(Arc::from("a"), 1);
        m1.insert(Arc::from("b"), 2);
        let mut m2: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m2.insert(Arc::from("b"), 2);
        m2.insert(Arc::from("a"), 1);
        assert_ne!(m1, m2, "same content different order must NOT be equal");
    }

    #[test]
    fn test_ordered_map_eq_same_order_is_equal() {
        let mut m1: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m1.insert(Arc::from("a"), 1);
        m1.insert(Arc::from("b"), 2);
        let mut m2: OrderedMap<Arc<str>, i32> = OrderedMap::new();
        m2.insert(Arc::from("a"), 1);
        m2.insert(Arc::from("b"), 2);
        assert_eq!(m1, m2, "same content same order must be equal");
    }
}
