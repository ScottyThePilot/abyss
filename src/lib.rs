//! Utilities that I don't want to write multiple times everywhere.

#![warn(
  absolute_paths_not_starting_with_crate,
  redundant_imports,
  redundant_lifetimes,
  future_incompatible,
  deprecated_in_future,
  missing_copy_implementations,
  missing_debug_implementations,
  unnameable_types,
  unreachable_pub
)]

#[cfg(feature = "serde")]
extern crate serde_core as serde;

pub mod externs {
  pub extern crate bytemuck;
  pub extern crate dunce;
  pub extern crate identity_hash;

  #[cfg(feature = "fixed")]
  pub extern crate fixed;

  #[cfg(feature = "log")]
  pub extern crate log;

  #[cfg(feature = "serde")]
  pub extern crate serde_core as serde;
}

#[macro_use]
pub mod macros;

pub mod cache;
pub mod error;
pub mod ids;
pub mod ptr;
pub mod sys;
pub mod lerp;
