//! Color constants and functions
//!
//! This module provides constants for colors that display well on Boppo, and functions for
//! modifying and working with colors.
//! Note that Boppo does not display black or other dark colors since when the LEDs are off
//! the buttons' natural coloring is grey.

pub use rgb;
pub use rgb::RGB8 as RGB;

/// #000000
pub const OFF: RGB = RGB::new(0, 0, 0);
/// #FFFFFF
pub const WHITE: RGB = RGB::new(255, 255, 255);
/// #7F7F7F
pub const GREY: RGB = RGB::new(127, 127, 127);
/// #404040
pub const DARK_GREY: RGB = RGB::new(64, 64, 64);

/// #FF0000
pub const RED: RGB = RGB::new(255, 0, 0);
/// #00FF00
pub const GREEN: RGB = RGB::new(0, 255, 0);
/// #0000FF
pub const BLUE: RGB = RGB::new(0, 0, 255);

/// #00FFFF
pub const CYAN: RGB = RGB::new(0, 255, 255);
/// #FFFF00
pub const YELLOW: RGB = RGB::new(255, 255, 0);
/// #FF00FF
pub const MAGENTA: RGB = RGB::new(255, 0, 255);

/// #FF7F00
pub const ORANGE: RGB = RGB::new(255, 127, 0);
/// #FF007F
pub const ROSE: RGB = RGB::new(255, 0, 127);
/// #7F00FF
pub const VIOLET: RGB = RGB::new(127, 0, 255);
/// #7FFF00
pub const CHARTREUSE: RGB = RGB::new(127, 255, 0);
/// #00FF7F
pub const SPRING_GREEN: RGB = RGB::new(0, 255, 127);
/// #007FFF
pub const AZURE: RGB = RGB::new(0, 127, 255);

/// #7F007F
pub const PURPLE: RGB = RGB::new(127, 0, 127);
/// #007F7F
pub const TEAL: RGB = RGB::new(0, 127, 127);

/// #FF7085
pub const PINK: RGB = RGB::new(255, 112, 133);

/// Blend two colors.
///
/// `percent` 0.0 returns `c1`, percent 1.0 returns `c2`.
/// Anything in between returns the weighted component average of `c1` and `c2`.
#[must_use]
pub const fn blend(c1: RGB, c2: RGB, percent_second: f32) -> RGB {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "`rhs_percent.clamp(0.0, 1.0)` ensures `f32::from(u8)*percent_second<=255`, u8 ensures non-negative."
    )]
    const fn mix_component(a: u8, b: u8, percent_second: f32) -> u8 {
        ((a as f32 * (1.0 - percent_second)).round() as u8)
            .saturating_add((b as f32 * percent_second).round() as u8)
    }

    let percent_second = percent_second.clamp(0.0, 1.0);

    RGB {
        r: mix_component(c1.r, c2.r, percent_second),
        g: mix_component(c1.g, c2.g, percent_second),
        b: mix_component(c1.b, c2.b, percent_second),
    }
}

/// Dim a color.
///
/// `percent`: 0.0 is OFF and 1.0 is `color` unchanged.
#[must_use]
pub const fn dim_to(color: RGB, percent: f32) -> RGB {
    blend(OFF, color, percent)
}

/// Brightness of a color.
///
/// `color::BLACK` has brightness 0.0, and `color::WHITE` has brightness 1.0
// A perceived brightness function might be useful too.
#[must_use]
pub fn brightness(color: RGB) -> f32 {
    let (r, g, b) = (f32::from(color.r), f32::from(color.g), f32::from(color.b));
    let magnitude = (r * r + g * g + b * b).sqrt();
    magnitude / 441.67296
}

/// Extension trait for useful operations on colors.
pub trait ColorExt {
    /// Blend two colors.
    ///
    /// `rhs_percent` 0.0 returns `self`, `rhs_percent` 1.0 returns `rhs`.
    /// Anything inbetween returns the weighted component average of `self` and `rhs`.
    #[must_use]
    fn blend(self, rhs: Self, rhs_percent: f32) -> Self;

    /// Dim a color.
    ///
    /// `percent`: 0.0 is [`OFF`] and 1.0 is `color` unchanged.
    #[must_use]
    fn dim_to(self, percent: f32) -> Self;

    /// Luminance of this color, not accounting for how the human eye perceives color.
    ///
    /// `color::BLACK` has luminance 0.0, and `color::WHITE` has luminance 1.0
    #[must_use]
    fn luminance(self) -> f32;
}

impl ColorExt for RGB {
    fn blend(self, rhs: Self, rhs_percent: f32) -> Self {
        blend(self, rhs, rhs_percent)
    }

    fn dim_to(self, percent: f32) -> Self {
        dim_to(self, percent)
    }

    fn luminance(self) -> f32 {
        brightness(self)
    }
}
