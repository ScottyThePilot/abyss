//! Utilities for non-blocking (non-async) caching/memoization of resources.

use std::convert::Infallible;

pub use super::{ResourceValidity, ResourceStatus};



/// Represents a resource which may be retrieved or computed.
///
/// The resource must implement [`ResourceValidity`] so that it may be invalidated.
pub trait Resource<Args>: ResourceValidity<Args> + Sized {
  /// Computes or retrieves the resource.
  fn get(args: Args) -> impl Future<Output = Self> + Send + Sync;
}

/// Represents a resource which may be retrieved or computed, fallibly.
///
/// The resource must implement [`ResourceValidity`] so that it may be invalidated.
pub trait ResourceFallible<Args>: ResourceValidity<Args> + Sized {
  /// The error type to emit when resource retrieval or computation fails.
  type Error;

  /// Attempts to compute or retrieve the resource.
  fn try_get(args: Args) -> impl Future<Output = Result<Self, Self::Error>> + Send + Sync;
}

impl<T, Args> ResourceFallible<Args> for T where T: Resource<Args> {
  type Error = Infallible;

  fn try_get(args: Args) -> impl Future<Output = Result<Self, Self::Error>> + Send + Sync {
    let future = Self::get(args);
    async { Ok(future.await) }
  }
}

/// Holds a potentially initialized resource.
#[derive(Debug, Clone)]
pub struct Cache<T> {
  state: Option<T>
}

impl<T> Cache<T> {
  /// Create a new cache, with no resource initialized.
  pub const fn new() -> Self {
    Cache { state: None }
  }

  /// Get the state (resource) from this cache as a reference.
  pub const fn get_state(&self) -> Option<&T> {
    self.state.as_ref()
  }

  /// Get the state (resource) from this cache as a mutable reference.
  pub const fn get_state_mut(&mut self) -> Option<&mut T> {
    self.state.as_mut()
  }

  /// Replace the contained state (resource) of this cache with the given value.
  pub const fn replace_state(&mut self, state: T) -> Option<T> {
    self.state.replace(state)
  }

  /// Replace the contained state (resource) of this cache with the given value, and return a mutable reference to it.
  pub fn insert_state(&mut self, state: T) -> &mut T {
    self.state.insert(state)
  }

  /// Clear the contained state (resource) of this cache.
  pub fn clear_state(&mut self) {
    self.state = None;
  }

  /// Get the resource status of this cache.
  pub fn get_status<Args>(&mut self, args: Args) -> ResourceStatus
  where T: ResourceValidity<Args> {
    match self.get_state() {
      None => ResourceStatus::Uninitialized,
      Some(state) => match state.is_valid(args) {
        true => ResourceStatus::InitializedValid,
        false => ResourceStatus::InitializedInvalid
      }
    }
  }

  /// Get the cached resource or retrieve/compute it.
  pub async fn get_or_init<Args>(&mut self, args: Args) -> &mut T
  where T: Resource<Args>, Args: Clone {
    get_or_init_resource(&mut self.state, args).await
  }

  /// Get the cached resource or retrieve/compute it, returning any errors encountered.
  pub async fn try_get_or_init<Args>(&mut self, args: Args) -> Result<&mut T, T::Error>
  where T: ResourceFallible<Args>, Args: Clone {
    try_get_or_init_resource(&mut self.state, args).await
  }
}

/// Get a resource or retrieve/compute it.
pub async fn get_or_init_resource<T, Args>(state_holder: &mut Option<T>, args: Args) -> &mut T
where T: Resource<Args>, Args: Clone {
  super::into_ok(try_get_or_init_resource(state_holder, args).await)
}

/// Get a resource or retrieve/compute it, returning any errors encountered.
pub async fn try_get_or_init_resource<T, Args>(state_holder: &mut Option<T>, args: Args) -> Result<&mut T, T::Error>
where T: ResourceFallible<Args> + ResourceValidity<Args>, Args: Clone {
  let state = match state_holder {
    None => T::try_get(args).await,
    Some(state) if !state.is_valid(args.clone()) => {
      *state_holder = None;
      T::try_get(args).await
    },
    Some(state) => return Ok(state)
  }?;

  Ok(state_holder.insert(state))
}
