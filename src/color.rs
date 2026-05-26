//! Color constants and functions
//!
//! This module provides constants for colors that display well on Boppo, and functions for
//! modifying and working with colors.
//! Note that Boppo does not display black or other dark colors since when the LEDs are off
//! the buttons' natural coloring is grey.

pub use rgb;
use rgb::ComponentMap;
pub use rgb::RGB8 as RGB;
pub use rgb::RGBA8 as RGBA;

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
pub const CHARTRUESE: RGB = RGB::new(127, 255, 0);
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

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "`rhs_percent.clamp(0.0, 1.0)` ensures `f32::from(u8)*percent_second<=255`, u8 ensures non-negative."
)]
const fn mix_component(a: u8, b: u8, percent_second: f32) -> u8 {
    let percent_second = percent_second.clamp(0.0, 1.0);
    ((a as f32 * (1.0 - percent_second)).round() as u8)
        .saturating_add((b as f32 * percent_second).round() as u8)
}

/// Extension trait for useful operations on colors.
pub trait Color: Copy + Default + PartialEq + rgb::bytemuck::NoUninit {
    /// BLACK or OFF for this Color.
    const OFF: Self;

    /// Weighted component average of two colors.
    ///
    /// `rhs_percent` 0.0 returns `self`, `rhs_percent` 1.0 returns `rhs`.
    /// Anything inbetween returns the weighted component average of `self` and `rhs`.
    #[must_use]
    fn weighted_average(self, rhs: Self, rhs_percent: f32) -> Self;

    /// Blend two colors together, according to their brightness or alpha.
    #[must_use]
    fn blend(self, rhs: Self) -> Self;

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

impl Color for RGB {
    const OFF: Self = OFF;

    fn weighted_average(self, rhs: Self, rhs_percent: f32) -> Self {
        RGB {
            r: mix_component(self.r, rhs.r, rhs_percent),
            g: mix_component(self.g, rhs.g, rhs_percent),
            b: mix_component(self.b, rhs.b, rhs_percent),
        }
    }

    fn blend(self, rhs: Self) -> Self {
        let self_luminance = self.luminance();
        let rhs_luminance = rhs.luminance();

        let self_a = self.with_alpha((self_luminance * 255.) as u8);
        let rhs_a = rhs.with_alpha((rhs_luminance * 255.) as u8);

        let result_a = alpha_blend(self_a, rhs_a);
        result_a.rgb().dim_to(f32::from(result_a.a) / 255.)
    }

    fn dim_to(self, percent: f32) -> Self {
        self.weighted_average(Self::OFF, percent)
    }

    fn luminance(self) -> f32 {
        let (r, g, b) = (f32::from(self.r), f32::from(self.g), f32::from(self.b));
        let magnitude = (r * r + g * g + b * b).sqrt();
        magnitude / 441.67296
    }
}

impl Color for RGBA {
    const OFF: Self = RGBA {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    fn weighted_average(self, rhs: Self, rhs_percent: f32) -> Self {
        RGBA {
            r: mix_component(self.r, rhs.r, rhs_percent),
            g: mix_component(self.g, rhs.g, rhs_percent),
            b: mix_component(self.b, rhs.b, rhs_percent),
            a: mix_component(self.a, rhs.a, rhs_percent),
        }
    }

    fn blend(self, rhs: Self) -> Self {
        alpha_blend(self, rhs)
    }

    fn dim_to(self, percent: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: mix_component(self.a, 0, percent),
        }
    }

    fn luminance(self) -> f32 {
        const MAX_MAGNITUDE_SQUARED: f32 = 255.0 * 255.0 * 3.0;

        let color_u8: [u8; 4] = rgb::bytemuck::bytes_of(&self).try_into().unwrap();
        let color = color_u8.map(f32::from);
        let magnitude = color[..3].iter().map(|c| c * c).sum::<f32>().sqrt();
        magnitude / MAX_MAGNITUDE_SQUARED.sqrt() * (color[3] / 255.0)
    }
}

/// Blends two [`RGBA`] together according to their alpha value, returning the result.
#[must_use]
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    reason = "Result is never signed, or above 255"
)]
pub fn alpha_blend(lhs: RGBA, rhs: RGBA) -> RGBA {
    let lhs = lhs.map(|c| f32::from(c) / 255.);
    let rhs = rhs.map(|c| f32::from(c) / 255.);
    let alpha_factor = lhs.a / rhs.a;
    let alpha_factor2 = (1.0 - lhs.a) / rhs.a;

    rgb::RGBA::<f32>::new(
        lhs.r * alpha_factor + rhs.r * alpha_factor2,
        lhs.g * alpha_factor + rhs.g * alpha_factor2,
        lhs.b * alpha_factor + rhs.b * alpha_factor2,
        1. - (1. - lhs.a) * (1. - rhs.a),
    )
    .map(|c| (c * 255.) as u8)
}
