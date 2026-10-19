//! Map structures keyed by [`Id`]s.

use std::collections::hash_map::{
  Entry as HashMapEntry,
  Keys as HashMapKeys,
  IntoKeys as HashMapIntoKeys,
  Values as HashMapValues,
  ValuesMut as HashMapValuesMut,
  IntoValues as HashMapIntoValues,
  Iter as HashMapIter,
  IterMut as HashMapIterMut,
  IntoIter as HashMapIntoIter,
  Drain as HashMapDrain,
  ExtractIf as HashMapExtractIf
};
use std::ops::Index;



use identity_hash::{IntMap, BuildIdentityHasher};

use super::{IdContext, Id};



pub type IdMapBuildHasher<T> = BuildIdentityHasher<Id<T>>;

/// A hashmap with [`Id`]s as opaque keys.
///
/// If you would like an [`IdContext`] to be paired with this, use [`IdMap`] instead.
pub type IdHashMap<T> = IntMap<Id<T>, T>;

/// A hashmap with [`Id`]s as opaque keys alongside an [`IdContext`] for producing them.
#[derive(Debug, Clone)]
pub struct IdMap<T> {
  context: IdContext<T>,
  map: IdHashMap<T>
}

impl<T> IdMap<T> {
  pub fn new() -> Self {
    IdMap::from_raw(IntMap::with_hasher(Default::default()))
  }

  pub fn with_capacity(capacity: usize) -> Self {
    IdMap::from_raw(IntMap::with_capacity_and_hasher(capacity, Default::default()))
  }

  pub const fn from_raw(map: IntMap<Id<T>, T>) -> Self {
    Self::from_raw_with_context(map, IdContext::new())
  }

  pub const fn from_raw_with_context(map: IntMap<Id<T>, T>, context: IdContext<T>) -> Self {
    IdMap { context, map }
  }

  pub fn capacity(&self) -> usize {
    self.map.capacity()
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }

  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  pub fn clear(&mut self) {
    self.context.current_id = 0;
    self.map.clear();
  }

  pub fn hasher(&self) -> &IdMapBuildHasher<T> {
    self.map.hasher()
  }

  pub fn reserve(&mut self, additional: usize) {
    self.map.reserve(additional);
  }

  pub fn shrink_to_fit(&mut self) {
    self.map.shrink_to_fit()
  }

  pub fn get(&self, id: Id<T>) -> Option<&T> {
    self.map.get(&id)
  }

  pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
    self.map.get_mut(&id)
  }

  pub fn get_disjoint_mut<const N: usize>(&mut self, ids: [Id<T>; N]) -> [Option<&mut T>; N] {
    self.map.get_disjoint_mut(ids.each_ref())
  }

  pub unsafe fn get_disjoint_mut_unchecked<const N: usize>(&mut self, ids: [Id<T>; N]) -> [Option<&mut T>; N] {
    unsafe { self.map.get_disjoint_unchecked_mut(ids.each_ref()) }
  }

  pub fn contains_id(&self, id: Id<T>) -> bool {
    self.map.contains_key(&id)
  }

  pub const fn get_current_id_raw(&self) -> u64 {
    self.context.get_current_id_raw()
  }

  pub const fn get_next_id(&mut self) -> Id<T> {
    self.context.get_next_id()
  }

  pub fn insert_new(&mut self, value: T) -> Id<T> {
    let id = self.get_next_id();
    let result = self.map.insert(id, value);
    debug_assert!(result.is_none());
    id
  }

  pub fn insert_new_with<F>(&mut self, f: F) -> Id<T>
  where F: FnOnce(Id<T>) -> T {
    let id = self.get_next_id();
    let result = self.map.insert(id, f(id));
    debug_assert!(result.is_none());
    id
  }

  pub fn insert(&mut self, id: Id<T>, value: T) -> Option<T> {
    self.map.insert(id, value)
  }

  pub fn entry(&mut self, id: Id<T>) -> HashMapEntry<'_, Id<T>, T> {
    self.map.entry(id)
  }

  pub fn entry_new(&mut self) -> HashMapEntry<'_, Id<T>, T> {
    let id = self.get_next_id();
    self.map.entry(id)
  }

  pub fn remove(&mut self, id: Id<T>) -> Option<T> {
    self.map.remove(&id)
  }

  pub fn remove_entry(&mut self, id: Id<T>) -> Option<(Id<T>, T)> {
    self.map.remove_entry(&id)
  }

  pub fn retain<F>(&mut self, mut f: F)
  where F: FnMut(Id<T>, &mut T) -> bool {
    self.map.retain(move |&id, v| f(id, v));
  }

  pub fn ids(&self) -> Ids<'_, T> {
    Ids { inner: self.map.keys() }
  }

  pub fn into_ids(self) -> IntoIds<T> {
    IntoIds { inner: self.map.into_keys() }
  }

  pub fn values(&self) -> Values<'_, T> {
    Values { inner: self.map.values() }
  }

  pub fn values_mut(&mut self) -> ValuesMut<'_, T> {
    ValuesMut { inner: self.map.values_mut() }
  }

  pub fn into_values(self) -> IntoValues<T> {
    IntoValues { inner: self.map.into_values() }
  }

  pub fn iter(&self) -> Iter<'_, T> {
    self.into_iter()
  }

  pub fn iter_mut(&mut self) -> IterMut<'_, T> {
    self.into_iter()
  }

  pub fn drain(&mut self) -> Drain<'_, T> {
    self.context.current_id = 0;
    Drain { inner: self.map.drain() }
  }

  pub fn extract_if<F>(&mut self, pred: F) -> ExtractIf<'_, T, F>
  where F: FnMut(&Id<T>, &mut T) -> bool {
    ExtractIf { inner: self.map.extract_if(pred) }
  }
}

impl<T> Default for IdMap<T> {
  fn default() -> Self {
    IdMap::new()
  }
}

impl<T> Index<Id<T>> for IdMap<T> {
  type Output = T;

  fn index(&self, id: Id<T>) -> &Self::Output {
    self.get(id).expect("no entry found for id")
  }
}

impl<T: Eq> Eq for IdMap<T> {}

impl<T: PartialEq> PartialEq for IdMap<T> {
  fn eq(&self, other: &Self) -> bool {
    self.map == other.map
  }
}

impl<'a, T> IntoIterator for &'a IdMap<T> {
  type Item = (Id<T>, &'a T);
  type IntoIter = Iter<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    Iter { inner: self.map.iter() }
  }
}

impl<'a, T> IntoIterator for &'a mut IdMap<T> {
  type Item = (Id<T>, &'a mut T);
  type IntoIter = IterMut<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    IterMut { inner: self.map.iter_mut() }
  }
}

impl<T> IntoIterator for IdMap<T> {
  type Item = (Id<T>, T);
  type IntoIter = IntoIter<T>;

  fn into_iter(self) -> Self::IntoIter {
    IntoIter { inner: self.map.into_iter() }
  }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for IdMap<T>
where T: serde::Serialize {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    self.map.serialize(serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for IdMap<T>
where T: serde::Deserialize<'de> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    struct IdMapVisitor<T>(std::marker::PhantomData<IdMap<T>>);

    impl<'de, T> serde::de::Visitor<'de> for IdMapVisitor<T>
    where T: serde::Deserialize<'de> {
      type Value = IdMap<T>;

      fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a map")
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where A: serde::de::MapAccess<'de> {
        let size = map.size_hint().unwrap_or(0).min(4096);
        let mut values = IntMap::with_capacity_and_hasher(size, Default::default());
        let mut current_id: u64 = 0;

        while let Some((key, value)) = map.next_entry::<Id<T>, T>()? {
          current_id = current_id.max(key.id);
          values.insert(key, value);
        };

        current_id += 1;

        Ok(IdMap {
          context: IdContext::with_current_id_raw(current_id),
          map: values
        })
      }
    }

    deserializer.deserialize_map(IdMapVisitor(std::marker::PhantomData))
  }
}



#[derive(Debug, Clone)]
pub struct Ids<'a, T> {
  inner: HashMapKeys<'a, Id<T>, T>
}

impl_iterator_map_traits!(
  { 'a, T }
  impl [Iterator, ExactSizeIterator, FusedIterator]
  for Ids<'a, T>, Id<T>, inner, |&id| id
);

#[derive(Debug)]
pub struct IntoIds<T> {
  inner: HashMapIntoKeys<Id<T>, T>
}

impl_iterator_map_traits!{
  { T }
  impl [Iterator, ExactSizeIterator, FusedIterator]
  for IntoIds<T>, Id<T>, inner, std::convert::identity
}

#[derive(Debug, Clone)]
pub struct Values<'a, T> {
  inner: HashMapValues<'a, Id<T>, T>
}

impl_iterator_map_traits!{
  { 'a, T }
  impl [Iterator, ExactSizeIterator, FusedIterator]
  for Values<'a, T>, &'a T, inner, std::convert::identity
}

#[derive(Debug)]
pub struct ValuesMut<'a, T> {
  inner: HashMapValuesMut<'a, Id<T>, T>
}

impl_iterator_map_traits!{
  { 'a, T }
  impl [Iterator, ExactSizeIterator, FusedIterator]
  for ValuesMut<'a, T>, &'a mut T, inner, std::convert::identity
}

#[derive(Debug)]
pub struct IntoValues<T> {
  inner: HashMapIntoValues<Id<T>, T>
}

impl_iterator_map_traits!{
  { T }
  impl [Iterator, ExactSizeIterator, FusedIterator]
  for IntoValues<T>, T, inner, std::convert::identity
}

#[derive(Debug, Clone)]
pub struct Iter<'a, T> {
  inner: HashMapIter<'a, Id<T>, T>
}

impl_iterator_map_traits!{
  { 'a, T }
  impl [Iterator, ExactSizeIterator, FusedIterator]
  for Iter<'a, T>, (Id<T>, &'a T), inner, |(&id, v)| (id, v)
}

#[derive(Debug)]
pub struct IterMut<'a, T> {
  inner: HashMapIterMut<'a, Id<T>, T>
}

impl_iterator_map_traits!{
  { 'a, T }
  impl [Iterator, ExactSizeIterator, FusedIterator]
  for IterMut<'a, T>, (Id<T>, &'a mut T), inner, |(&id, v)| (id, v)
}

#[derive(Debug)]
pub struct IntoIter<T> {
  inner: HashMapIntoIter<Id<T>, T>
}

impl_iterator_map_traits!{
  { T }
  impl [Iterator, ExactSizeIterator, FusedIterator]
  for IntoIter<T>, (Id<T>, T), inner, std::convert::identity
}

#[derive(Debug)]
pub struct Drain<'a, T> {
  inner: HashMapDrain<'a, Id<T>, T>
}

impl_iterator_map_traits!{
  { 'a, T }
  impl [Iterator, ExactSizeIterator, FusedIterator]
  for Drain<'a, T>, (Id<T>, T), inner, std::convert::identity
}

#[derive(Debug)]
pub struct ExtractIf<'a, T, F> {
  inner: HashMapExtractIf<'a, Id<T>, T, F>
}

impl_iterator_map_traits!{
  { 'a, T, F: FnMut(&Id<T>, &mut T) -> bool }
  impl [Iterator, FusedIterator]
  for ExtractIf<'a, T, F>, (Id<T>, T), inner, std::convert::identity
}
