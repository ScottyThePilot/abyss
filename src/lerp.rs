//! Utilities for linear interpolation.



pub trait Lerp<Factor = Self> {
  type Output;

  fn lerp(operands: [Self; 2], factor: Factor) -> Self::Output where Self: Sized;
}

impl Lerp for f32 {
  type Output = f32;

  fn lerp([lhs, rhs]: [Self; 2], factor: f32) -> Self::Output {
    lhs.mul_add(1.0 - factor, rhs * factor)
  }
}

impl Lerp for f64 {
  type Output = f64;

  fn lerp([lhs, rhs]: [Self; 2], factor: f64) -> Self::Output {
    lhs.mul_add(1.0 - factor, rhs * factor)
  }
}

pub trait LerpSlice<Factor> {
  type Element: Lerp<Factor, Output = Self::Element> + Clone;

  fn lerp_slice(&self, factor: Factor) -> Option<Self::Element>;
}

impl<T, Factor> LerpSlice<Factor> for [T]
where T: Lerp<Factor, Output = T> + Clone, Factor: LerpSliceFactor {
  type Element = T;

  fn lerp_slice(&self, factor: Factor) -> Option<Self::Element> {
    let (lhs, rhs, factor) = factor.normalize(self.len())?;
    if lhs == rhs {
      self.get(lhs).cloned()
    } else {
      let lhs = self.get(lhs).cloned()?;
      let rhs = self.get(rhs).cloned()?;
      Some(T::lerp([lhs, rhs], factor))
    }
  }
}

impl<T, Factor, const N: usize> LerpSlice<Factor> for [T; N]
where T: Lerp<Factor, Output = T> + Clone, Factor: LerpSliceFactor {
  type Element = T;

  fn lerp_slice(&self, factor: Factor) -> Option<Self::Element> {
    self.as_slice().lerp_slice(factor)
  }
}

impl<T, Factor> LerpSlice<Factor> for Vec<T>
where T: Lerp<Factor, Output = T> + Clone, Factor: LerpSliceFactor {
  type Element = T;

  fn lerp_slice(&self, factor: Factor) -> Option<Self::Element> {
    self.as_slice().lerp_slice(factor)
  }
}

macro_rules! impl_lerp_slice_delegate {
  (<$Slice:ident> $Type:ty) => (
    impl<$Slice: LerpSlice<Factor>, Factor> LerpSlice<Factor> for $Type {
      type Element = $Slice::Element;

      fn lerp_slice(&self, factor: Factor) -> Option<Self::Element> {
        S::lerp_slice(self, factor)
      }
    }
  );
}

impl_lerp_slice_delegate!(<S> &S);
impl_lerp_slice_delegate!(<S> &mut S);
impl_lerp_slice_delegate!(<S> std::boxed::Box<S>);
impl_lerp_slice_delegate!(<S> std::rc::Rc<S>);
impl_lerp_slice_delegate!(<S> std::sync::Arc<S>);



pub trait LerpSliceFactor: Sized {
  fn normalize(self, len: usize) -> Option<(usize, usize, Self)>;
}

impl LerpSliceFactor for f32 {
  fn normalize(self, len: usize) -> Option<(usize, usize, Self)> {
    if len == 0 { return None };
    let factor = self.clamp(0.0, 1.0) * len.checked_sub(1)? as f32;
    let lhs = factor.floor() as usize;
    let rhs = factor.ceil() as usize;
    let factor = factor.rem_euclid(1.0);
    Some((lhs, rhs, factor))
  }
}

impl LerpSliceFactor for f64 {
  fn normalize(self, len: usize) -> Option<(usize, usize, Self)> {
    if len == 0 { return None };
    let factor = self.clamp(0.0, 1.0) * len.checked_sub(1)? as f64;
    let lhs = factor.floor() as usize;
    let rhs = factor.ceil() as usize;
    let factor = factor.rem_euclid(1.0);
    Some((lhs, rhs, factor))
  }
}

#[cfg(feature = "fixed")]
mod fixed {
  use super::{Lerp, LerpSliceFactor};

  use fixed::{FixedU8, FixedU16, FixedU32, FixedU64, FixedU128, FixedI8, FixedI16, FixedI32, FixedI64, FixedI128};
  use fixed::types::extra::*;

  macro_rules! impl_lerp_fixed {
    ($Fixed:ident, $Trait:ident) => (
      impl<Frac, FracFactor: $Trait> Lerp<$Fixed<FracFactor>> for $Fixed<Frac> {
        type Output = $Fixed<Frac>;

        fn lerp([lhs, rhs]: [Self; 2], factor: $Fixed<FracFactor>) -> Self::Output {
          $Fixed::lerp(factor, lhs, rhs)
        }
      }
    );
  }

  impl_lerp_fixed!(FixedU8, LeEqU8);
  impl_lerp_fixed!(FixedU16, LeEqU16);
  impl_lerp_fixed!(FixedU32, LeEqU32);
  impl_lerp_fixed!(FixedU64, LeEqU64);
  impl_lerp_fixed!(FixedU128, LeEqU128);
  impl_lerp_fixed!(FixedI8, LeEqU8);
  impl_lerp_fixed!(FixedI16, LeEqU16);
  impl_lerp_fixed!(FixedI32, LeEqU32);
  impl_lerp_fixed!(FixedI64, LeEqU64);
  impl_lerp_fixed!(FixedI128, LeEqU128);

  macro_rules! impl_lerp_slice_factor_fixed {
    ($Fixed:ident, $FracMax:ident, $FracMin:ident) => (
      impl LerpSliceFactor for $Fixed<$FracMax> {
        fn normalize(self, len: usize) -> Option<(usize, usize, Self)> {
          let factor = self.wide_mul($Fixed::<$FracMin>::from_num(len.checked_sub(1)?));
          let lhs = factor.floor().to_num::<usize>();
          let rhs = factor.ceil().to_num::<usize>();
          let factor = factor.rem_euclid_int(1).saturating_to_num::<Self>();
          Some((lhs, rhs, factor))
        }
      }
    );
  }

  impl_lerp_slice_factor_fixed!(FixedU8, U8, U0);
  impl_lerp_slice_factor_fixed!(FixedU16, U16, U0);
  impl_lerp_slice_factor_fixed!(FixedU32, U32, U0);
  impl_lerp_slice_factor_fixed!(FixedU64, U64, U0);
}

mod compound {
  use super::Lerp;

  impl<T, Factor, const N: usize> Lerp<[Factor; N]> for [T; 2]
  where [T; 2]: Lerp<Factor::Compound>, Factor: IntoCompound<N> {
    type Output = <[T; 2] as Lerp<Factor::Compound>>::Output;

    fn lerp(operands: [Self; 2], factor: [Factor; N]) -> Self::Output where Self: Sized {
      Lerp::lerp(operands, Factor::into_compound(factor))
    }
  }

  impl<T, FactorInner, FactorOuter> Lerp<Compound<FactorInner, FactorOuter>> for [T; 2]
  where T: Lerp<FactorInner>, T::Output: Lerp<FactorOuter>, FactorInner: Copy {
    type Output = <T::Output as Lerp<FactorOuter>>::Output;

    fn lerp(operands: [Self; 2], factor: Compound<FactorInner, FactorOuter>) -> Self::Output {
      Lerp::lerp(operands.map(|operands| Lerp::lerp(operands, factor.inner)), factor.outer)
    }
  }

  #[allow(unnameable_types)]
  #[derive(Debug, Clone, Copy)]
  pub struct Compound<FactorInner, FactorOuter> {
    inner: FactorInner,
    outer: FactorOuter
  }

  impl<FactorInner, FactorOuter> Compound<FactorInner, FactorOuter> {
    const fn new(inner: FactorInner, outer: FactorOuter) -> Self {
      Compound { inner, outer }
    }
  }

  #[allow(unnameable_types)]
  pub trait IntoCompound<const N: usize> {
    type Compound;

    fn into_compound(values: [Self; N]) -> Self::Compound where Self: Sized;
  }

  impl<T> IntoCompound<2> for T {
    type Compound = Compound<T, T>;

    fn into_compound([t0, t1]: [Self; 2]) -> Self::Compound {
      Compound::new(t1, t0)
    }
  }

  impl<T> IntoCompound<3> for T {
    type Compound = Compound<Compound<T, T>, T>;

    fn into_compound([t0, t1, t2]: [Self; 3]) -> Self::Compound {
      Compound::new(Compound::new(t2, t1), t0)
    }
  }

  impl<T> IntoCompound<4> for T {
    type Compound = Compound<Compound<Compound<T, T>, T>, T>;

    fn into_compound([t0, t1, t2, t3]: [Self; 4]) -> Self::Compound {
      Compound::new(Compound::new(Compound::new(t3, t2), t1), t0)
    }
  }
}



#[cfg(test)]
mod tests {
  use super::*;

  const fn abs_diff(a: f32, b: f32) -> f32 {
    (a - b).abs()
  }

  #[test]
  fn test_2d_lerp() {
    let mut matrix: [[f32; 2]; 2] = Default::default();

    matrix[0][0] = 25.0;
    matrix[1][0] = 10.0;
    matrix[0][1] = 13.7;
    matrix[1][1] = 40.2;

    let center_calculated = matrix.into_iter().flatten().sum::<f32>() / 4.0;
    let center_interpolated = Lerp::lerp(matrix, [0.5; 2]);

    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0]), 25.0, "at [0, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0]), 10.0, "at [1, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0]), 13.7, "at [0, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0]), 40.2, "at [1, 1]");

    assert!(abs_diff(center_calculated, center_interpolated) < 0.001, "at center");
  }

  #[test]
  fn test_3d_lerp() {
    let mut matrix: [[[f32; 2]; 2]; 2] = Default::default();

    matrix[0][0][0] = 25.0;
    matrix[1][0][0] = 10.0;
    matrix[0][1][0] = 13.7;
    matrix[1][1][0] = 40.2;
    matrix[0][0][1] = 20.6;
    matrix[1][0][1] = 40.9;
    matrix[0][1][1] = 84.4;
    matrix[1][1][1] = 11.1;

    let center_calculated = matrix.into_iter().flatten().flatten().sum::<f32>() / 8.0;
    let center_interpolated = Lerp::lerp(matrix, [0.5; 3]);

    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 0.0]), 25.0, "at [0, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 0.0]), 10.0, "at [1, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 0.0]), 13.7, "at [0, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 0.0]), 40.2, "at [1, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 1.0]), 20.6, "at [0, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 1.0]), 40.9, "at [1, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 1.0]), 84.4, "at [0, 1, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 1.0]), 11.1, "at [1, 1, 1]");

    assert!(abs_diff(center_calculated, center_interpolated) < 0.001, "at center");
  }

  #[test]
  fn test_4d_lerp() {
    let mut matrix: [[[[f32; 2]; 2]; 2]; 2] = Default::default();

    matrix[0][0][0][0] = 25.0;
    matrix[1][0][0][0] = 10.0;
    matrix[0][1][0][0] = 13.7;
    matrix[1][1][0][0] = 40.2;
    matrix[0][0][1][0] = 20.6;
    matrix[1][0][1][0] = 40.9;
    matrix[0][1][1][0] = 84.4;
    matrix[1][1][1][0] = 11.1;
    matrix[0][0][0][1] = 65.7;
    matrix[1][0][0][1] = 17.0;
    matrix[0][1][0][1] = 28.8;
    matrix[1][1][0][1] = 99.5;
    matrix[0][0][1][1] = 34.6;
    matrix[1][0][1][1] = 94.2;
    matrix[0][1][1][1] = 22.9;
    matrix[1][1][1][1] = 20.2;

    let center_calculated = matrix.into_iter().flatten().flatten().flatten().sum::<f32>() / 16.0;
    let center_interpolated = Lerp::lerp(matrix, [0.5; 4]);

    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 0.0, 0.0]), 25.0, "at [0, 0, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 0.0, 0.0]), 10.0, "at [1, 0, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 0.0, 0.0]), 13.7, "at [0, 1, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 0.0, 0.0]), 40.2, "at [1, 1, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 1.0, 0.0]), 20.6, "at [0, 0, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 1.0, 0.0]), 40.9, "at [1, 0, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 1.0, 0.0]), 84.4, "at [0, 1, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 1.0, 0.0]), 11.1, "at [1, 1, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 0.0, 1.0]), 65.7, "at [0, 0, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 0.0, 1.0]), 17.0, "at [1, 0, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 0.0, 1.0]), 28.8, "at [0, 1, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 0.0, 1.0]), 99.5, "at [1, 1, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 1.0, 1.0]), 34.6, "at [0, 0, 1, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 1.0, 1.0]), 94.2, "at [1, 0, 1, 1]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 1.0, 1.0]), 22.9, "at [0, 1, 1, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 1.0, 1.0]), 20.2, "at [1, 1, 1, 1]");

    assert!(abs_diff(center_calculated, center_interpolated) < 0.001, "at center");
  }
}
