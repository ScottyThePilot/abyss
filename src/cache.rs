//! Utilities for caching/memoization of resources.

pub mod blocking;
pub mod nonblocking;

use std::convert::Infallible;



/// Quickly determines the validity of a resource.
pub trait ResourceValidity<Args> {
  /// Returns true if the resource is still valid,
  /// false if it is invalid and should be re-retrieved or re-computed.
  fn is_valid(&self, args: Args) -> bool;
}

/// The status of a resource cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceStatus {
  /// No resource has been retrieved or computed yet.
  Uninitialized,
  /// The resource has been retrieved or computed, and is still valid.
  InitializedValid,
  /// The resource has been retrieved or computed, but is now invalid.
  InitializedInvalid
}



fn into_ok<T>(result: Result<T, Infallible>) -> T {
  match result {
    Ok(value) => value,
    Err(error) => match error {}
  }
}
