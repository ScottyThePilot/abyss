extern crate abyss;
extern crate fixed;

use fixed::types::*;

use abyss::lerp::{Lerp, LerpSlice};

fn main() {
  let a = U16F16::from_num(1.23456);
  let b = U16F16::from_num(2.34567);
  let c = U16F16::from_num(3.45678);
  let d = U16F16::from_num(4.56789);

  dbg!(a, b, c, d);

  let l = [a, b, c, d];

  dbg!(Lerp::lerp([a, b], U0F32::saturating_from_num(0.0)));
  dbg!(Lerp::lerp([a, b], U0F32::saturating_from_num(0.5)));
  dbg!(Lerp::lerp([b, c], U0F32::saturating_from_num(0.5)));
  dbg!(Lerp::lerp([c, d], U0F32::saturating_from_num(0.5)));
  dbg!(Lerp::lerp([c, d], U0F32::saturating_from_num(1.0)));

  dbg!(LerpSlice::lerp_slice(&l, U0F32::saturating_from_num(0.0)).unwrap());
  dbg!(LerpSlice::lerp_slice(&l, U0F32::saturating_from_num(0.5)).unwrap());
  dbg!(LerpSlice::lerp_slice(&l, U0F32::saturating_from_num(1.0)).unwrap());
}
