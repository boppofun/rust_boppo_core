//! Utilities for interpolation between two values.
use crate::color;

/// A type that can be interpolated between two values (e.g. blended or transitioned)
pub trait Interpolatable {
    /// Return Self `percent` of the way from `a` (0.0) to `b` (1.0)
    fn interpolate(percent: f32, a: &Self, b: &Self) -> Self;
}

impl Interpolatable for f32 {
    fn interpolate(percent: f32, a: &Self, b: &Self) -> Self {
        lerp(*a, *b, percent)
    }
}

impl Interpolatable for color::RGB {
    fn interpolate(percent: f32, a: &Self, b: &Self) -> Self {
        color::blend(*a, *b, percent)
    }
}

/// Linear interpolation between two f32 values.
///
/// `percent` should be a value between 0.0 and 1.0, with 0.0 being `v1` and 1.0 being `v2`.
#[must_use]
pub fn lerp(v1: f32, v2: f32, percent: f32) -> f32 {
    v2 * percent + v1 * (1. - percent)
}
