//! Utility macros.

#[doc(hidden)]
pub fn map_fold<T, U, A>(
  mut map: impl FnMut(T) -> U,
  mut f: impl FnMut(A, U) -> A,
) -> impl FnMut(A, T) -> A {
  move |acc, value| f(acc, map(value))
}

#[macro_export]
macro_rules! impl_iterator_map_traits {
  ({ $($t:tt)* } impl [] for $Type:ty, $Item:ty, $inner:ident, $map_func:expr $(,)?) => ();
  ({ $($t:tt)* } impl [$Trait:ident] for $Type:ty, $Item:ty, $inner:ident, $map_func:expr $(,)?) => (
    $crate::impl_iterator_map_trait!({ $($t)* } impl $Trait for $Type, $Item, $inner, $map_func);
  );
  ({ $($t:tt)* } impl [$Trait:ident, $($Traits:ident),* $(,)?] for $Type:ty, $Item:ty, $inner:ident, $map_func:expr $(,)?) => (
    $crate::impl_iterator_map_trait!({ $($t)* } impl $Trait for $Type, $Item, $inner, $map_func);
    $crate::impl_iterator_map_traits!({ $($t)* } impl [$($Traits),*] for $Type, $Item, $inner, $map_func);
  );
}

#[macro_export]
macro_rules! impl_iterator_map_trait {
  ({ $($t:tt)* } impl Iterator for $Type:ty, $Item:ty, $inner:ident, $map_func:expr $(,)?) => (
    impl<$($t)*> core::iter::Iterator for $Type {
      type Item = $Item;

      $crate::impl_iterator_map_methods!(Iterator, $inner, $map_func);
    }
  );
  ({ $($t:tt)* } impl DoubleEndedIterator for $Type:ty, $Item:ty, $inner:ident, $map_func:expr $(,)?) => (
    impl<$($t)*> core::iter::DoubleEndedIterator for $Type {
      $crate::impl_iterator_map_methods!(DoubleEndedIterator, $inner, $map_func);
    }
  );
  ({ $($t:tt)* } impl ExactSizeIterator for $Type:ty, $Item:ty, $inner:ident $(, $map_func:expr $(,)?)?) => (
    impl<$($t)*> core::iter::ExactSizeIterator for $Type {
      $crate::impl_iterator_map_methods!(ExactSizeIterator, $inner $(, $map_func)?);
    }
  );
  ({ $($t:tt)* } impl FusedIterator for $Type:ty $(, $Item:ty, $inner:ident, $map_func:expr $(,)?)?) => (
    impl<$($t)*> core::iter::FusedIterator for $Type {}
  );
}

#[macro_export]
macro_rules! impl_iterator_map_methods {
  (Iterator, $inner:ident, $map_func:expr $(,)?) => (
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
      self.$inner.next().map($map_func)
    }

    fn fold<__A, __F>(self, init: __A, f: __F) -> __A
    where __F: FnMut(__A, Self::Item) -> __A {
      self.$inner.fold(init, $crate::macros::map_fold($map_func, f))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
      self.$inner.size_hint()
    }
  );
  (DoubleEndedIterator, $inner:ident, $map_func:expr $(,)?) => (
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
      self.$inner.next_back().map($map_func)
    }

    fn rfold<__A, __F>(self, init: __A, f: __F) -> __A
    where __F: FnMut(__A, Self::Item) -> __A {
      self.$inner.rfold(init, $crate::macros::map_fold($map_func, f))
    }
  );
  (ExactSizeIterator, $inner:ident $(, $map_func:expr $(,)?)?) => (
    fn len(&self) -> usize {
      self.$inner.len()
    }
  );
}
