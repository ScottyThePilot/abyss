//! Utilities relating to various types of pointers.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};



/// Allows conversion of values to `*const T` pointers.
pub trait AsPtr {
  /// The value that should be pointed to once converted to a raw pointer.
  type Pointee: ?Sized;

  /// Convert this value to a raw pointer.
  fn as_ptr(&self) -> *const Self::Pointee;

  /// Convert this value to a raw pointer, with the type info erased.
  #[inline]
  fn as_ptr_erased(&self) -> *const () {
    self.as_ptr().cast::<()>()
  }

  /// Check the equality of this value pointer-wise with another value.
  #[inline]
  fn ptr_eq(&self, other: &impl AsPtr) -> bool {
    <*const ()>::eq(&self.as_ptr_erased(), &other.as_ptr_erased())
  }

  /// Compare this value pointer-wise with another value.
  #[inline]
  fn ptr_cmp(&self, other: &impl AsPtr) -> Ordering {
    <*const ()>::cmp(&self.as_ptr_erased(), &other.as_ptr_erased())
  }

  /// Hash this value pointer-wise.
  #[inline]
  fn ptr_hash(&self, state: &mut impl Hasher) {
    <*const ()>::hash(&self.as_ptr_erased(), state);
  }
}

impl<T: ?Sized> AsPtr for *const T {
  type Pointee = T;

  fn as_ptr(&self) -> *const Self::Pointee {
    *self
  }
}

impl<T: ?Sized> AsPtr for *mut T {
  type Pointee = T;

  fn as_ptr(&self) -> *const Self::Pointee {
    *self
  }
}

impl<T: ?Sized> AsPtr for std::boxed::Box<T> {
  type Pointee = T;

  fn as_ptr(&self) -> *const Self::Pointee {
    std::boxed::Box::as_ptr(self)
  }
}

impl<T: ?Sized> AsPtr for std::rc::Rc<T> {
  type Pointee = T;

  fn as_ptr(&self) -> *const Self::Pointee {
    std::rc::Rc::as_ptr(self)
  }
}

impl<T: ?Sized> AsPtr for std::rc::Weak<T> {
  type Pointee = T;

  fn as_ptr(&self) -> *const Self::Pointee {
    std::rc::Weak::as_ptr(self)
  }
}

impl<T: ?Sized> AsPtr for std::sync::Arc<T> {
  type Pointee = T;

  fn as_ptr(&self) -> *const Self::Pointee {
    std::sync::Arc::as_ptr(self)
  }
}

impl<T: ?Sized> AsPtr for std::sync::Weak<T> {
  type Pointee = T;

  fn as_ptr(&self) -> *const Self::Pointee {
    std::sync::Weak::as_ptr(self)
  }
}



/// Allows conversion of values to `*const T` pointers.
pub trait AsPtrMut: AsPtr {
  /// Convert this value to a raw pointer.
  fn as_mut_ptr(&mut self) -> *mut Self::Pointee;

  /// Convert this value to a raw pointer, with the type info erased.
  fn as_mut_ptr_erased(&mut self) -> *mut () {
    self.as_mut_ptr().cast::<()>()
  }
}

impl<T: ?Sized> AsPtrMut for *mut T {
  fn as_mut_ptr(&mut self) -> *mut Self::Pointee {
    *self
  }
}

impl<T: ?Sized> AsPtrMut for std::boxed::Box<T> {
  fn as_mut_ptr(&mut self) -> *mut Self::Pointee {
    std::boxed::Box::as_mut_ptr(self)
  }
}
