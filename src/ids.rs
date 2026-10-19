//! Opaque ID generators and types.

pub extern crate identity_hash;

pub mod map;

use identity_hash::IdentityHashable;

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};



#[repr(transparent)]
pub struct Id<F: ?Sized = ()> {
  id: u64,
  family: PhantomData<F>
}

impl<F: ?Sized> Id<F> {
  pub const fn from_id_raw(id: u64) -> Self {
    Id { id, family: PhantomData }
  }

  pub const fn into_id_raw(self) -> u64 {
    self.id
  }

  pub const fn cast<U>(self) -> Id<U> {
    Id::from_id_raw(self.into_id_raw())
  }
}

impl<F: ?Sized> fmt::Debug for Id<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let family_name = std::any::type_name::<F>();

    f.debug_tuple("Id")
      .field(&self.id)
      .field(&format_args!("{family_name}"))
      .finish()
  }
}

impl<F: ?Sized> Clone for Id<F> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<F: ?Sized> Copy for Id<F> {}

impl<F: ?Sized> Eq for Id<F> {}

impl<F: ?Sized> PartialEq for Id<F> {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<F: ?Sized> PartialOrd for Id<F> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(u64::cmp(&self.id, &other.id))
  }
}

impl<F: ?Sized> Ord for Id<F> {
  fn cmp(&self, other: &Self) -> Ordering {
    u64::cmp(&self.id, &other.id)
  }
}

impl<F: ?Sized> Hash for Id<F> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.id);
  }
}

unsafe impl<F: ?Sized> Send for Id<F> {}
unsafe impl<F: ?Sized> Sync for Id<F> {}

impl<F: ?Sized> IdentityHashable for Id<F> {}

#[cfg(feature = "serde")]
impl<F: ?Sized> serde::Serialize for Id<F> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    self.id.serialize(serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, F: ?Sized> serde::Deserialize<'de> for Id<F> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    u64::deserialize(deserializer).map(Id::from_id_raw)
  }
}



/// A context for spawning unique IDs.
#[repr(transparent)]
pub struct IdContext<F: ?Sized = ()> {
  current_id: u64,
  family: PhantomData<F>
}

impl<F: ?Sized> IdContext<F> {
  /// Creates a new ID context.
  pub const fn new() -> IdContext<F> {
    Self::with_current_id_raw(0)
  }

  /// Creates a new ID context with an from which it should start counting from.
  pub const fn with_current_id_raw(current_id: u64) -> IdContext<F> {
    IdContext {
      current_id,
      family: PhantomData
    }
  }

  pub const fn into_current_id_raw(self) -> u64 {
    self.current_id
  }

  pub const fn get_current_id_raw(&self) -> u64 {
    self.current_id
  }

  /// Spawns the next unique ID for this context.
  pub const fn get_next_id(&mut self) -> Id<F> {
    let id = self.current_id;
    self.current_id += 1;
    Id::from_id_raw(id)
  }
}

impl<F: ?Sized> fmt::Debug for IdContext<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let family_name = std::any::type_name::<F>();

    f.debug_tuple("IdContext")
      .field(&self.get_current_id_raw())
      .field(&format_args!("{family_name}"))
      .finish()
  }
}

impl<F: ?Sized> Clone for IdContext<F> {
  fn clone(&self) -> Self {
    IdContext::with_current_id_raw(self.get_current_id_raw())
  }
}

impl<F: ?Sized> Default for IdContext<F> {
  fn default() -> IdContext<F> {
    IdContext::new()
  }
}

unsafe impl<F: ?Sized> Send for IdContext<F> {}
unsafe impl<F: ?Sized> Sync for IdContext<F> {}

#[cfg(feature = "serde")]
impl<'de, F: ?Sized> serde::Deserialize<'de> for IdContext<F> {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    struct Visitor<F: ?Sized> {
      family: PhantomData<F>
    }

    impl<'de, F: ?Sized> serde::de::Visitor<'de> for Visitor<F> {
      type Value = IdContext<F>;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "tuple struct IdContext")
      }

      #[inline]
      fn visit_newtype_struct<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        <u64 as serde::Deserialize>::deserialize(deserializer)
          .map(IdContext::with_current_id_raw)
      }

      #[inline]
      fn visit_seq<Seq: serde::de::SeqAccess<'de>>(self, mut seq: Seq) -> Result<Self::Value, Seq::Error> {
        seq.next_element::<u64>()?
          .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))
          .map(IdContext::with_current_id_raw)
      }
    }

    deserializer.deserialize_newtype_struct("IdContext", Visitor { family: PhantomData })
  }
}

#[cfg(feature = "serde")]
impl<F: ?Sized> serde::Serialize for IdContext<F> {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_newtype_struct("IdContext", &self.get_current_id_raw())
  }
}



/// An atomic context for spawning unique IDs.
/// This is just like `IdContext`, but operates atomically and can be shared between threads.
#[repr(transparent)]
pub struct IdContextAtomic<F: ?Sized = ()> {
  current_id: AtomicU64,
  family: PhantomData<F>
}

impl<F: ?Sized> IdContextAtomic<F> {
  /// Creates a new ID context.
  pub const fn new() -> IdContextAtomic<F> {
    Self::with_current_id_raw(0)
  }

  /// Creates a new ID context with an from which it should start counting from.
  pub const fn with_current_id_raw(current_id: u64) -> IdContextAtomic<F> {
    IdContextAtomic {
      current_id: AtomicU64::new(current_id),
      family: PhantomData
    }
  }

  pub const fn into_current_id_raw(self) -> u64 {
    self.current_id.into_inner()
  }

  pub fn get_current_id_raw(&self) -> u64 {
    self.current_id.load(AtomicOrdering::Relaxed)
  }

  pub fn get_next_id(&self) -> Id<F> {
    Id::from_id_raw(self.current_id.fetch_add(1, AtomicOrdering::Relaxed))
  }
}

impl<F: ?Sized> fmt::Debug for IdContextAtomic<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let family_name = std::any::type_name::<F>();

    f.debug_tuple("IdContextAtomic")
      .field(&self.get_current_id_raw())
      .field(&format_args!("{family_name}"))
      .finish()
  }
}

impl<F: ?Sized> Clone for IdContextAtomic<F> {
  fn clone(&self) -> Self {
    IdContextAtomic::with_current_id_raw(self.get_current_id_raw())
  }
}

impl<F: ?Sized> Default for IdContextAtomic<F> {
  fn default() -> IdContextAtomic<F> {
    IdContextAtomic::new()
  }
}

unsafe impl<F: ?Sized> Send for IdContextAtomic<F> {}
unsafe impl<F: ?Sized> Sync for IdContextAtomic<F> {}

#[cfg(feature = "serde")]
impl<'de, F: ?Sized> serde::Deserialize<'de> for IdContextAtomic<F> {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    struct Visitor<F: ?Sized> {
      family: PhantomData<F>
    }

    impl<'de, F: ?Sized> serde::de::Visitor<'de> for Visitor<F> {
      type Value = IdContextAtomic<F>;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "tuple struct IdContextAtomic")
      }

      #[inline]
      fn visit_newtype_struct<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        <u64 as serde::Deserialize>::deserialize(deserializer)
          .map(IdContextAtomic::with_current_id_raw)
      }

      #[inline]
      fn visit_seq<Seq: serde::de::SeqAccess<'de>>(self, mut seq: Seq) -> Result<Self::Value, Seq::Error> {
        seq.next_element::<u64>()?
          .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))
          .map(IdContextAtomic::with_current_id_raw)
      }
    }

    deserializer.deserialize_newtype_struct("IdContextAtomic", Visitor { family: PhantomData })
  }
}

#[cfg(feature = "serde")]
impl<F: ?Sized> serde::Serialize for IdContextAtomic<F> {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_newtype_struct("IdContextAtomic", &self.get_current_id_raw())
  }
}
