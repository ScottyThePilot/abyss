extern crate abyss;

use abyss::lerp::{Lerp, LerpSlice};

fn main() {
  let a = 1.23456_f64;
  let b = 2.34567_f64;
  let c = 3.45678_f64;
  let d = 4.56789_f64;

  dbg!(a, b, c, d);

  let l = [a, b, c, d];

  dbg!(Lerp::lerp([a, b], 0.0));
  dbg!(Lerp::lerp([a, b], 0.5));
  dbg!(Lerp::lerp([b, c], 0.5));
  dbg!(Lerp::lerp([c, d], 0.5));
  dbg!(Lerp::lerp([c, d], 1.0));

  dbg!(LerpSlice::lerp_slice(&l, 0.0).unwrap());
  dbg!(LerpSlice::lerp_slice(&l, 0.5).unwrap());
  dbg!(LerpSlice::lerp_slice(&l, 1.0).unwrap());
}
