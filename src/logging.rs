//! A set of logger helper traits for easier error handling while logging.
//!
//! Some code was adapted from or took inspiration from `fern` `0.7.1`, under the MIT license.

#![cfg(feature = "log")]

mod output;
mod rich;

pub use self::output::*;
pub use self::rich::*;
