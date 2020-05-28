// Copyright 2019 The Aljabar Developers. For a full listing of authors,
// refer to the Cargo.toml file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
//! The super generic super experimental linear algebra library.
//!
//! This library serves the dual purpose of being an experimental API for
//! future rust linear algebra libraries as well as a test of rustc's strength
//! in compiling a number of in development features, such as const generics
//! and specialization.
//!
//! It is not the specific goal of this project to be useful in any sense, but
//! hopefully it will end up being roughly compatible with cgmath.
//!
//! Aljabar has no benchmarks and cannot make any performance guarantees at the
//! moment.
//!

#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(trivial_bounds)]
#![feature(maybe_uninit_ref)]

use core::{
    cmp::PartialOrd,
    fmt,
    hash::{Hash, Hasher},
    iter::{FromIterator, Product},
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ops::{
        Add, AddAssign, Deref, DerefMut, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub,
        SubAssign,
    },
};

#[cfg(feature = "mint")]
use mint;

#[cfg(feature = "rand")]
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[cfg(feature = "serde")]
use serde::{
    de::{Error, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

mod array;
mod vector;
mod matrix;
mod point;
mod rotation;
pub mod row_view;

pub use array::*;
pub use vector::*;
pub use point::*;
use row_view::*;
pub use rotation::*;
pub use matrix::*;

/// Defines the additive identity for `Self`.
pub trait Zero {
    /// Returns the additive identity of `Self`.
    fn zero() -> Self;

    /// Returns true if the value is the additive identity.
    fn is_zero(&self) -> bool;
}

macro_rules! impl_zero {
    // Default $zero to '0' if not provided.
    (
        $type:ty
    ) => {
        impl_zero! { $type, 0 }
    };
    // Main impl.
    (
        $type:ty,
        $zero:expr
    ) => {
        impl Zero for $type {
            fn zero() -> Self {
                $zero
            }

            fn is_zero(&self) -> bool {
                *self == $zero
            }
        }
    };
}

impl_zero! { bool, false }
impl_zero! { f32, 0.0 }
impl_zero! { f64, 0.0 }
impl_zero! { i8 }
impl_zero! { i16 }
impl_zero! { i32 }
impl_zero! { i64 }
impl_zero! { i128 }
impl_zero! { isize }
impl_zero! { u8 }
impl_zero! { u16 }
impl_zero! { u32 }
impl_zero! { u64 }
impl_zero! { u128 }
impl_zero! { usize }

/// Defines the multiplicative identity element for `Self`.
///
/// For Matrices, `one` is an alias for the unit matrix.
pub trait One {
    /// Returns the multiplicative identity for `Self`.
    fn one() -> Self;

    /// Returns true if the value is the multiplicative identity.
    fn is_one(&self) -> bool;
}

macro_rules! impl_one {
    // Default $one to '1' if not provided.
    (
        $type:ty
    ) => {
        impl_one! { $type, 1 }
    };
    // Main impl.
    (
        $type:ty,
        $one:expr
    ) => {
        impl One for $type {
            fn one() -> Self {
                $one
            }

            fn is_one(&self) -> bool {
                *self == $one
            }
        }
    };
}

impl_one! { bool, true }
impl_one! { f32, 1.0 }
impl_one! { f64, 1.0 }
impl_one! { i8 }
impl_one! { i16 }
impl_one! { i32 }
impl_one! { i64 }
impl_one! { i128 }
impl_one! { isize }
impl_one! { u8 }
impl_one! { u16 }
impl_one! { u32 }
impl_one! { u64 }
impl_one! { u128 }
impl_one! { usize }

/// Values that are [real numbers](https://en.wikipedia.org/wiki/Real_number#Axiomatic_approach).
pub trait Real
where
    Self: Sized,
    Self: Add<Output = Self>,
    Self: Sub<Output = Self>,
    Self: Mul<Output = Self>,
    Self: Div<Output = Self>,
    Self: Neg<Output = Self>,
{
    fn sqrt(self) -> Self;

    fn mul2(self) -> Self;

    fn div2(self) -> Self;

    fn abs(self) -> Self;
}

impl Real for f32 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    fn mul2(self) -> Self {
        2.0 * self
    }

    fn div2(self) -> Self {
        self / 2.0
    }

    fn abs(self) -> Self {
        self.abs()
    }
}

impl Real for f64 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    fn mul2(self) -> Self {
        2.0 * self
    }

    fn div2(self) -> Self {
        self / 2.0
    }

    fn abs(self) -> Self {
        self.abs()
    }
}

/// A point in 1-dimensional space.
pub type Point1<T> = Point<T, 1>;

/// A point in 2-dimensional space.
pub type Point2<T> = Point<T, 2>;

/// A point in 3-dimensional space.
pub type Point3<T> = Point<T, 3>;

/// A point in 4-dimensional space.
pub type Point4<T> = Point<T, 4>;

/// A point in 5-dimensional space.
pub type Point5<T> = Point<T, 5>;

/// Vectors that can be added together and multiplied by scalars form a
/// VectorSpace.
///
/// If a `Vector` implements `Add` and `Sub` and its scalar implements `Mul` and
/// `Div`, then that vector is part of a `VectorSpace`.
pub trait VectorSpace
where
    Self: Sized + Zero,
    Self: Add<Self, Output = Self>,
    Self: Sub<Self, Output = Self>,
    Self: Mul<<Self as VectorSpace>::Scalar, Output = Self>,
    Self: Div<<Self as VectorSpace>::Scalar, Output = Self>,
{
    // I only need Div, but I felt like I had to add them all...
    type Scalar: Add<Self::Scalar, Output = Self::Scalar>
        + Sub<Self::Scalar, Output = Self::Scalar>
        + Mul<Self::Scalar, Output = Self::Scalar>
        + Div<Self::Scalar, Output = Self::Scalar>;

    fn lerp(self, other: Self, amount: Self::Scalar) -> Self;
}

impl<T, const N: usize> VectorSpace for Vector<T, { N }>
where
    T: Clone + Zero,
    T: Add<T, Output = T>,
    T: Sub<T, Output = T>,
    T: Mul<T, Output = T>,
    T: Div<T, Output = T>,
{
    type Scalar = T;

    fn lerp(self, other: Self, amount: Self::Scalar) -> Self {
        self.clone() + ((other - self) * amount)
    }
}

/// A type with a distance function between two values.
pub trait MetricSpace: Sized {
    type Metric;

    /// Returns the distance squared between the two values.
    fn distance2(self, other: Self) -> Self::Metric;
}

/// A metric spaced where the metric is a real number.
pub trait RealMetricSpace: MetricSpace
where
    Self::Metric: Real,
{
    /// Returns the distance between the two values.
    fn distance(self, other: Self) -> Self::Metric {
        self.distance2(other).sqrt()
    }
}

impl<T> RealMetricSpace for T
where
    T: MetricSpace,
    <T as MetricSpace>::Metric: Real,
{
}

impl<T, const N: usize> MetricSpace for Vector<T, { N }>
where
    Self: InnerSpace,
{
    type Metric = <Self as VectorSpace>::Scalar;

    fn distance2(self, other: Self) -> Self::Metric {
        (other - self).magnitude2()
    }
}

/// Vector spaces that have an inner (also known as "dot") product.
pub trait InnerSpace: VectorSpace
where
    Self: Clone,
    Self: MetricSpace<Metric = <Self as VectorSpace>::Scalar>,
{
    /// Return the inner (also known as "dot") product.
    fn dot(self, other: Self) -> Self::Scalar;

    /// Returns the squared length of the value.
    fn magnitude2(self) -> Self::Scalar {
        self.clone().dot(self)
    }

    /// Returns the [reflection](https://en.wikipedia.org/wiki/Reflection_(mathematics))
    /// of the current vector with respect to the given surface normal. The surface normal
    /// must be of length 1 for the return value to be correct. The current vector is
    /// interpreted as pointing toward the surface, and does not need to be normalized.
    fn reflect(self, surface_normal: Self) -> Self {
        let a = surface_normal.clone() * self.clone().dot(surface_normal);
        self - (a.clone() + a)
    }
}

/// Defines an InnerSpace where the Scalar is a real number. Automatically
/// implemented.
pub trait RealInnerSpace: InnerSpace
where
    Self: Clone,
    Self: MetricSpace<Metric = <Self as VectorSpace>::Scalar>,
    <Self as VectorSpace>::Scalar: Real,
{
    /// Returns the length of the vector.
    fn magnitude(self) -> Self::Scalar {
        self.clone().dot(self).sqrt()
    }

    /// Returns a vector with the same direction and a magnitude of `1`.
    fn normalize(self) -> Self
    where
        Self::Scalar: One,
    {
        self.normalize_to(<Self::Scalar as One>::one())
    }

    /// Returns a vector with the same direction and a given magnitude.
    fn normalize_to(self, magnitude: Self::Scalar) -> Self {
        self.clone() * (magnitude / self.magnitude())
    }

    /// Returns the
    /// [vector projection](https://en.wikipedia.org/wiki/Vector_projection)
    /// of the current inner space projected onto the supplied argument.
    fn project_on(self, other: Self) -> Self {
        other.clone() * (self.dot(other.clone()) / other.magnitude2())
    }
}

impl<T> RealInnerSpace for T
where
    T: InnerSpace,
    <T as VectorSpace>::Scalar: Real,
{
}

impl<T, const N: usize> InnerSpace for Vector<T, { N }>
where
    T: Clone + Zero,
    T: Add<T, Output = T>,
    T: Sub<T, Output = T>,
    T: Mul<T, Output = T>,
    T: Div<T, Output = T>,
    // TODO: Remove this add assign bound. This is purely for ease of
    // implementation.
    T: AddAssign<T>,
    Self: Clone,
{
    fn dot(self, rhs: Self) -> T {
        let mut lhs = MaybeUninit::new(self);
        let mut rhs = MaybeUninit::new(rhs);
        let mut sum = <T as Zero>::zero();
        let lhsp: *mut MaybeUninit<T> = unsafe { mem::transmute(&mut lhs) };
        let rhsp: *mut MaybeUninit<T> = unsafe { mem::transmute(&mut rhs) };
        for i in 0..N {
            sum += unsafe {
                lhsp.add(i).replace(MaybeUninit::uninit()).assume_init()
                    * rhsp.add(i).replace(MaybeUninit::uninit()).assume_init()
            };
        }
        sum
    }
}

/// A 1-by-1 square matrix.
pub type Mat1x1<T> = Matrix<T, 1, 1>;

/// A 2-by-2 square matrix.
pub type Mat2x2<T> = Matrix<T, 2, 2>;

/// A 3-by-3 square matrix.
pub type Mat3x3<T> = Matrix<T, 3, 3>;

/// A 4-by-4 square matrix.
pub type Mat4x4<T> = Matrix<T, 4, 4>;

#[cfg(test)]
mod tests {
    use super::*;
    use approx::{abs_diff_eq, AbsDiffEq};

    impl<T: AbsDiffEq, const N: usize> AbsDiffEq for Vector<T, { N }>
    where
        T::Epsilon: Copy,
    {
        type Epsilon = T::Epsilon;

        fn default_epsilon() -> T::Epsilon {
            T::default_epsilon()
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: T::Epsilon) -> bool {
            self.iter()
                .zip(other.iter())
                .all(|(x, y)| T::abs_diff_eq(x, y, epsilon))
        }
    }

    impl<T: AbsDiffEq, const N: usize, const M: usize> AbsDiffEq for Matrix<T, { N }, { M }>
    where
        T::Epsilon: Copy,
    {
        type Epsilon = T::Epsilon;

        fn default_epsilon() -> T::Epsilon {
            T::default_epsilon()
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: T::Epsilon) -> bool {
            self.column_iter()
                .zip(other.column_iter())
                .all(|(x, y)| Vector::<T, { N }>::abs_diff_eq(x, y, epsilon))
        }
    }

    /*
    #[test]
    fn test_permutation() {
        let p1 = Permutation::unit();
        let p2 = Permutation([0usize, 1, 2]);
        let p3 = Permutation([1usize, 2, 0]);
        let v = vector!(1.0f64, 2.0, 3.0);
        assert_eq!(p1, p2);
        assert_eq!(v, p3 * (p3 * (p3 * v)));
    }

    #[test]
    fn test_permutation_parity() {
        let p1 = Permutation::<4>::unit();
        let p2 = Permutation([3usize, 1, 2, 0]);
        let p3 = Permutation([2usize, 3, 1, 0]);
        assert!(!p1.odd_parity());
        assert!(p2.odd_parity());
        assert!(p3.odd_parity());
    }
    */

    #[test]
    fn test_vec_zero() {
        let a = Vector3::<u32>::zero();
        assert_eq!(a, Vector3::<u32>::from([0, 0, 0]));
    }

    #[test]
    fn test_vec_index() {
        let a = Vector1::<u32>::from([0]);
        assert_eq!(a[0], 0);
        let mut b = Vector2::<u32>::from([1, 2]);
        b[1] += 3;
        assert_eq!(b[1], 5);
    }

    #[test]
    fn test_vec_eq() {
        let a = Vector1::<u32>::from([0]);
        let b = Vector1::<u32>::from([1]);
        let c = Vector1::<u32>::from([0]);
        let d = [0u32];
        assert_ne!(a, b);
        assert_eq!(a, c);
        assert_eq!(a, &d); // No blanket impl on T for deref... why? infinite loops?
    }

    #[test]
    fn test_vec_addition() {
        let a = Vector1::<u32>::from([0]);
        let b = Vector1::<u32>::from([1]);
        let c = Vector1::<u32>::from([2]);
        assert_eq!(a + b, b);
        assert_eq!(b + b, c);
        // We shouldn't need to have to test more dimensions, but we shall test
        // one more.
        let a = Vector2::<u32>::from([0, 1]);
        let b = Vector2::<u32>::from([1, 2]);
        let c = Vector2::<u32>::from([1, 3]);
        let d = Vector2::<u32>::from([2, 5]);
        assert_eq!(a + b, c);
        assert_eq!(b + c, d);
        let mut c = Vector2::<u32>::from([1, 3]);
        let d = Vector2::<u32>::from([2, 5]);
        c += d;
        let e = Vector2::<u32>::from([3, 8]);
        assert_eq!(c, e);
    }

    #[test]
    fn test_vec_subtraction() {
        let mut a = Vector1::<u32>::from([3]);
        let b = Vector1::<u32>::from([1]);
        let c = Vector1::<u32>::from([2]);
        assert_eq!(a - c, b);
        a -= b;
        assert_eq!(a, c);
    }

    #[test]
    fn test_vec_negation() {
        let a = Vector4::<i32>::from([1, 2, 3, 4]);
        let b = Vector4::<i32>::from([-1, -2, -3, -4]);
        assert_eq!(-a, b);
    }

    #[test]
    fn test_vec_scale() {
        let a = Vector4::<f32>::from([2.0, 4.0, 2.0, 4.0]);
        let b = Vector4::<f32>::from([4.0, 8.0, 4.0, 8.0]);
        let c = Vector4::<f32>::from([1.0, 2.0, 1.0, 2.0]);
        assert_eq!(a * 2.0, b);
        assert_eq!(a / 2.0, c);
    }

    #[test]
    fn test_vec_cross() {
        let a = vector!(1isize, 2isize, 3isize);
        let b = vector!(4isize, 5isize, 6isize);
        let r = vector!(-3isize, 6isize, -3isize);
        assert_eq!(a.cross(b), r);
    }

    #[test]
    fn test_vec_distance() {
        let a = Vector1::<f32>::from([0.0]);
        let b = Vector1::<f32>::from([1.0]);
        assert_eq!(a.distance2(b), 1.0);
        let a = Vector1::<f32>::from([0.0]);
        let b = Vector1::<f32>::from([2.0]);
        assert_eq!(a.distance2(b), 4.0);
        assert_eq!(a.distance(b), 2.0);
        let a = Vector2::<f32>::from([0.0, 0.0]);
        let b = Vector2::<f32>::from([1.0, 1.0]);
        assert_eq!(a.distance2(b), 2.0);
    }

    #[test]
    fn test_vec_normalize() {
        let a = vector!(5.0);
        assert_eq!(a.clone().magnitude(), 5.0);
        let a_norm = a.normalize();
        assert_eq!(a_norm, vector!(1.0));
    }

    #[test]
    fn test_vec_transpose() {
        let v = vector!(1i32, 2, 3, 4);
        let m = Matrix::<i32, 1, 4>::from([vector!(1i32), vector!(2), vector!(3), vector!(4)]);
        assert_eq!(v.transpose(), m);
    }

    #[test]
    fn test_from_fn() {
        let indices: Vector<usize, 10> = vector!(0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9);
        assert_eq!(Vector::<usize, 10>::from_fn(|i| i), indices);
    }

    #[test]
    fn test_decompose() {
        let a = matrix![[2.0, 1.0], [-1.0f64, 1.0]];
        let b = vector!(2.0f64, 5.0);
        let lu = a.lu().unwrap();

        assert_eq!(a * lu.solve(b), b);
    }

    #[test]
    fn test_vec_map() {
        let int = vector!(1i32, 0, 1, 1, 0, 1, 1, 0, 0, 0);
        let boolean = vector!(true, false, true, true, false, true, true, false, false, false);
        assert_eq!(int.map(|i| i != 0), boolean);
    }

    #[test]
    fn test_vec_from_iter() {
        let v = vec![1i32, 2, 3, 4];
        let vec = Vector::<i32, 4>::from_iter(v);
        assert_eq!(vec, vector![1i32, 2, 3, 4])
    }

    #[test]
    fn test_vec_into_iter() {
        let v = vector!(1i32, 2, 3, 4);
        let vec: Vec<i32> = v.into_iter().collect();
        assert_eq!(vec, vec![1i32, 2, 3, 4])
    }

    #[test]
    fn test_vec_indexed_map() {
        let boolean = vector!(true, false, true, true, false, true, true, false, false, false);
        let indices = vector!(0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9);
        assert_eq!(boolean.indexed_map(|i, _| i), indices);
    }

    // Does not compile.
    /*
    #[test]
    fn test_vec_first() {
        let a = Vector2::<i32>::from([ 1, 2 ]);
        let b = Vector3::<i32>::from([ 1, 2, 3 ]);
        let c = b.first::<2_usize>();
        assert_eq!(a, c);
    }
    */

    #[test]
    fn test_mat_identity() {
        let unit = matrix![[1u32, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1],];
        assert_eq!(Matrix::<u32, 4, 4>::one(), unit);
    }

    #[test]
    fn test_mat_negation() {
        let neg_unit = matrix![
            [-1i32, 0, 0, 0],
            [0, -1, 0, 0],
            [0, 0, -1, 0],
            [0, 0, 0, -1],
        ];
        assert_eq!(-Matrix::<i32, 4, 4>::one(), neg_unit);
    }

    #[test]
    fn test_mat_add() {
        let a = matrix![[matrix![[1u32]]]];
        let b = matrix![[matrix![[10u32]]]];
        let c = matrix![[matrix![[11u32]]]];
        assert_eq!(a + b, c);
    }

    #[test]
    fn test_mat_scalar_mult() {
        let a = Matrix::<f32, 2, 2>::from([vector!(0.0, 1.0), vector!(0.0, 2.0)]);
        let b = Matrix::<f32, 2, 2>::from([vector!(0.0, 2.0), vector!(0.0, 4.0)]);
        assert_eq!(a * 2.0, b);
    }

    #[test]
    fn test_mat_mult() {
        let a = Matrix::<f32, 2, 2>::from([vector!(0.0, 0.0), vector!(1.0, 0.0)]);
        let b = Matrix::<f32, 2, 2>::from([vector!(0.0, 1.0), vector!(0.0, 0.0)]);
        assert_eq!(a * b, matrix![[1.0, 0.0], [0.0, 0.0],]);
        assert_eq!(b * a, matrix![[0.0, 0.0], [0.0, 1.0],]);
        // Basic example:
        let a: Matrix<usize, 1, 1> = matrix![[1]];
        let b: Matrix<usize, 1, 1> = matrix![[2]];
        let c: Matrix<usize, 1, 1> = matrix![[2]];
        assert_eq!(a * b, c);
        // Removing the type signature here caused the compiler to crash.
        // Since then I've been wary.
        let a = Matrix::<f32, 3, 3>::from([
            vector!(1.0, 0.0, 0.0),
            vector!(0.0, 1.0, 0.0),
            vector!(0.0, 0.0, 1.0),
        ]);
        let b = a.clone();
        let c = a * b;
        assert_eq!(
            c,
            matrix![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],]
        );
        // Here is another random example I found online.
        let a: Matrix<i32, 3, 3> = matrix![[0, -3, 5], [6, 1, -4], [2, 3, -2],];
        let b: Matrix<i32, 3, 3> = matrix![[-1, 0, -3], [4, 5, 1], [2, 6, -2]];
        let c: Matrix<i32, 3, 3> = matrix![[-2, 15, -13], [-10, -19, -9], [6, 3, 1]];
        assert_eq!(a * b, c);
    }

    #[test]
    fn test_mat_index() {
        let m: Matrix<i32, 2, 2> = matrix![[0, 2], [1, 3],];
        assert_eq!(m[(0, 0)], 0);
        assert_eq!(m[0][0], 0);
        assert_eq!(m[(1, 0)], 1);
        assert_eq!(m[0][1], 1);
        assert_eq!(m[(0, 1)], 2);
        assert_eq!(m[1][0], 2);
        assert_eq!(m[(1, 1)], 3);
        assert_eq!(m[1][1], 3);
    }

    #[test]
    fn test_mat_transpose() {
        assert_eq!(
            Matrix::<i32, 1, 2>::from([vector!(1), vector!(2)]).transpose(),
            Matrix::<i32, 2, 1>::from([vector!(1, 2)])
        );
        assert_eq!(
            matrix![[1, 2], [3, 4],].transpose(),
            matrix![[1, 3], [2, 4],]
        );
    }

    #[test]
    fn test_square_matrix() {
        let a: Matrix<i32, 3, 3> = matrix![[5, 0, 0], [0, 8, 12], [0, 0, 16],];
        let diag: Vector<i32, 3> = vector!(5, 8, 16);
        assert_eq!(a.diagonal(), diag);
    }

    #[test]
    fn test_readme_code() {
        let a = vector!(0u32, 1, 2, 3);
        assert_eq!(a, Vector::<u32, 4>::from([0u32, 1, 2, 3]));

        let b = Vector::<f32, 7>::from([0.0f32, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let c = Vector::<f32, 7>::from([1.0f32, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]) * 0.5;
        assert_eq!(
            b + c,
            Vector::<f32, 7>::from([0.5f32, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5])
        );

        let a = vector!(1i32, 1);
        let b = vector!(5i32, 5);
        assert_eq!(a.distance2(b), 32); // distance method not implemented.
        assert_eq!((b - a).magnitude2(), 32); // magnitude method not implemented.

        let a = vector!(1.0f32, 1.0);
        let b = vector!(5.0f32, 5.0);
        const CLOSE: f32 = 5.65685424949;
        assert_eq!(a.distance(b), CLOSE); // distance is implemented.
        assert_eq!((b - a).magnitude(), CLOSE); // magnitude is implemented.

        // Vector normalization is also supported for floating point scalars.
        assert_eq!(
            vector!(0.0f32, 20.0, 0.0).normalize(),
            vector!(0.0f32, 1.0, 0.0)
        );

        let _a = Matrix::<f32, 3, 3>::from([
            vector!(1.0, 0.0, 0.0),
            vector!(0.0, 1.0, 0.0),
            vector!(0.0, 0.0, 1.0),
        ]);
        let _b: Matrix<i32, 3, 3> = matrix![[0, -3, 5], [6, 1, -4], [2, 3, -2]];

        assert_eq!(
            matrix![[1i32, 0, 0,], [0, 2, 0], [0, 0, 3],].diagonal(),
            vector!(1i32, 2, 3)
        );

        assert_eq!(
            matrix![[1i32, 0, 0, 0], [0, 2, 0, 0], [0, 0, 3, 0], [0, 0, 0, 4]].diagonal(),
            vector!(1i32, 2, 3, 4)
        );
    }

    #[test]
    fn test_mat_map() {
        let int = matrix![[1i32, 0], [1, 1], [0, 1], [1, 0], [0, 0]];
        let boolean = matrix![
            [true, false],
            [true, true],
            [false, true],
            [true, false],
            [false, false]
        ];
        assert_eq!(int.map(|i| i != 0), boolean);
    }

    #[test]
    fn test_mat_from_iter() {
        let v = vec![1i32, 2, 3, 4];
        let mat = Matrix::<i32, 2, 2>::from_iter(v);
        assert_eq!(mat, matrix![[1i32, 2], [3, 4]].transpose())
    }

    #[test]
    fn test_mat_invert() {
        assert!(Mat2x2::<f64>::one().invert().unwrap() == Mat2x2::<f64>::one());

        // Example taken from cgmath:

        let a: Mat2x2<f64> = matrix![[1.0f64, 2.0f64], [3.0f64, 4.0f64],];
        let identity: Mat2x2<f64> = Mat2x2::<f64>::one();
        abs_diff_eq!(
            a.invert().unwrap(),
            matrix![[-2.0f64, 1.0f64], [1.5f64, -0.5f64]]
        );

        abs_diff_eq!(a.invert().unwrap() * a, identity);
        abs_diff_eq!(a * a.invert().unwrap(), identity);
        assert!(matrix![[0.0f64, 2.0f64], [0.0f64, 5.0f64]]
            .invert()
            .is_none());
    }

    #[test]
    fn test_mat_determinant() {
        assert_eq!(Mat2x2::<f64>::one().determinant(), f64::one());
        /*
        assert_eq!(
            matrix![[3.0f64, 8.0f64], [4.0f64, 6.0f64]].invert().unwrap(),
            matrix![[3.0f64, 8.0f64], [4.0f64, 6.0f64]]
        );
        */
        assert_eq!(
            matrix![[3.0f64, 8.0f64], [4.0f64, 6.0f64]].determinant(),
            -14.0f64
        );
        assert_eq!(
            matrix![[-2.0f64, 1.0f64], [1.5f64, -0.5f64]].determinant(),
            -0.5f64
        );
        assert_eq!(
            matrix![[6.0f64, 1.0, 1.0], [4.0, -2.0, 5.0], [2.0, 8.0, 7.0]].determinant(),
            -306.0f64
        );
    }

    #[test]
    fn test_vec_macro_constructor() {
        let v: Vector<f32, 0> = vector![];
        assert!(v.is_empty());

        let v = vector![1];
        assert_eq!(1, v[0]);

        let v = vector![1, 2, 3, 4, 5, 6, 7, 8, 9, 10,];
        for i in 0..10 {
            assert_eq!(i + 1, v[i]);
        }
    }

    #[test]
    fn test_mat_macro_constructor() {
        let m: Matrix<f32, 0, 0> = matrix![];
        assert!(m.is_empty());

        let m = matrix![[1]];
        assert_eq!(1, m[0][0]);

        let m = matrix![[1, 2], [3, 4], [5, 6],];
        assert_eq!(
            m,
            Matrix::<u32, 3, 2>::from([
                Vector::<u32, 3>::from([1, 3, 5]),
                Vector::<u32, 3>::from([2, 4, 6])
            ])
        );
    }

    #[test]
    fn test_vec_swizzle() {
        let v: Vector<f32, 1> = Vector::<f32, 1>::from([1.0]);
        assert_eq!(1.0, v.x());

        let v: Vector<f32, 2> = Vector::<f32, 2>::from([1.0, 2.0]);
        assert_eq!(1.0, v.x());
        assert_eq!(2.0, v.y());

        let v: Vector<f32, 3> = Vector::<f32, 3>::from([1.0, 2.0, 3.0]);
        assert_eq!(1.0, v.x());
        assert_eq!(2.0, v.y());
        assert_eq!(3.0, v.z());

        let v: Vector<f32, 4> = Vector::<f32, 4>::from([1.0, 2.0, 3.0, 4.0]);
        assert_eq!(1.0, v.x());
        assert_eq!(2.0, v.y());
        assert_eq!(3.0, v.z());
        assert_eq!(4.0, v.w());

        let v: Vector<f32, 5> = Vector::<f32, 5>::from([1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(1.0, v.x());
        assert_eq!(2.0, v.y());
        assert_eq!(3.0, v.z());
        assert_eq!(4.0, v.w());
    }

    #[test]
    fn test_vec_reflect() {
        // Incident straight on to the surface.
        let v = vector!(1, 0);
        let n = vector!(-1, 0);
        let r = v.reflect(n);
        assert_eq!(r, vector!(-1, 0));

        // Incident at 45 degree angle to the surface.
        let v = vector!(1, 1);
        let n = vector!(-1, 0);
        let r = v.reflect(n);
        assert_eq!(r, vector!(-1, 1));
    }

    #[test]
    fn test_rotation() {
        let rot = Orthonormal::<f32, 3>::from(Euler {
            x: 0.0,
            y: 0.0,
            z: core::f32::consts::FRAC_PI_2,
        });
        assert_eq!(rot.rotate_vector(vector![1.0f32, 0.0, 0.0]).y(), 1.0);
        let v = vector![1.0f32, 0.0, 0.0];
        let q1 = Quaternion::from(Euler {
            x: 0.0,
            y: 0.0,
            z: core::f32::consts::FRAC_PI_2,
        });
        assert_eq!(q1.rotate_vector(v).normalize().y(), 1.0);
    }
}

#[cfg(all(feature = "mint", test))]
mod mint_tests {
    use super::*;

    #[test]
    fn point2_roundtrip() {
        let alj1 = point![1, 2];
        let mint: mint::Point2<u32> = alj1.into();
        let alj2: Point<u32, 2> = mint.into();
        assert_eq!(alj1, alj2);
    }

    #[test]
    fn point3_roundtrip() {
        let alj1 = point![1, 2, 3];
        let mint: mint::Point3<u32> = alj1.into();
        let alj2: Point<u32, 3> = mint.into();
        assert_eq!(alj1, alj2);
    }

    #[test]
    fn vector2_roundtrip() {
        let alj1 = vector![1, 2];
        let mint: mint::Vector2<u32> = alj1.into();
        let alj2: Vector<u32, 2> = mint.into();
        assert_eq!(alj1, alj2);
    }

    #[test]
    fn vector3_roundtrip() {
        let alj1 = vector![1, 2, 3];
        let mint: mint::Vector3<u32> = alj1.into();
        let alj2: Vector<u32, 3> = mint.into();
        assert_eq!(alj1, alj2);
    }

    #[test]
    fn vector4_roundtrip() {
        let alj1 = vector![1, 2, 3, 4];
        let mint: mint::Vector4<u32> = alj1.into();
        let alj2: Vector<u32, 4> = mint.into();
        assert_eq!(alj1, alj2);
    }

    #[test]
    fn quaternion_roundtrip() {
        let alj1 = Quaternion::new(1, 2, 3, 4);
        let mint: mint::Quaternion<u32> = alj1.into();
        let alj2: Quaternion<u32> = mint.into();
        assert_eq!(alj1, alj2);
    }

    #[test]
    fn matrix2x2_roundtrip() {
        let alj1 = matrix![[1, 2], [3, 4]];
        let mint_col: mint::ColumnMatrix2<u32> = alj1.into();
        let mint_row: mint::RowMatrix2<u32> = alj1.into();
        let alj2: Matrix<u32, 2, 2> = mint_col.into();
        let alj3: Matrix<u32, 2, 2> = mint_row.into();
        assert_eq!(alj1, alj2);
        assert_eq!(alj1, alj3);
    }

    #[test]
    fn matrix3x2_roundtrip() {
        let alj1 = matrix![[1, 2], [3, 4], [5, 6]];
        let mint_col: mint::ColumnMatrix3x2<u32> = alj1.into();
        let mint_row: mint::RowMatrix3x2<u32> = alj1.into();
        let alj2: Matrix<u32, 3, 2> = mint_col.into();
        let alj3: Matrix<u32, 3, 2> = mint_row.into();
        assert_eq!(alj1, alj2);
        assert_eq!(alj1, alj3);
    }

    #[test]
    fn matrix3x4_roundtrip() {
        let alj1 = matrix![[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]];
        let mint_col: mint::ColumnMatrix3x4<u32> = alj1.into();
        let mint_row: mint::RowMatrix3x4<u32> = alj1.into();
        let alj2: Matrix<u32, 3, 4> = mint_col.into();
        let alj3: Matrix<u32, 3, 4> = mint_row.into();
        assert_eq!(alj1, alj2);
        assert_eq!(alj1, alj3);
    }
}
