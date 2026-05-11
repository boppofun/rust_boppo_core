use std::ops::{Add, Div, Mul, Sub};

use rgb::{ColorComponentMap, ComponentMap};

use crate::{
    Buttons, Lights, MainFramebuffer,
    color::{Color, RGB, RGBA},
};

/// On "off-screen" buffer storing a color for each light on the tablet.
///
/// These colors can be modifed with a simlar API as `[Buttons]` but will
/// not have any affect until `[flush]` is called.
#[derive(Clone, PartialEq)]
#[must_use]
pub struct Framebuffer<C: Color = RGB> {
    /// A color for each Light on the tablet.
    /// The ordering is the same as [`Lights`].
    pub colors: [C; Lights::COUNT],
}

impl<C: Color + Default> Default for Framebuffer<C> {
    fn default() -> Self {
        Framebuffer {
            colors: [C::default(); Lights::COUNT],
        }
    }
}

impl<C: Color> Framebuffer<C> {
    /// Create a new Framebuffer with all pixels set to OFF (Black).
    pub fn new() -> Self {
        Self {
            colors: [C::OFF; Lights::COUNT],
        }
    }

    /// Set every light in `selection` to `color`.
    pub fn set_color<B: Into<Lights>>(&mut self, selection: B, color: C) {
        let selection = selection.into();
        for lights_idx in selection.indices() {
            self.colors[lights_idx] = color;
        }
    }

    /// Set every button in `buttons` to the corresponding color in `colors`, up until either all
    /// buttons selected in `buttons` have been used, or all colors in `colors` have been used.
    pub fn set_buttons_colors(&mut self, buttons: Buttons, colors: impl IntoIterator<Item = C>) {
        for (button, color) in buttons.buttons().zip(colors) {
            self.set_color(button, color);
        }
    }

    /// Set every light to the corresponding color in `colors`.
    pub fn set_all_colors(&mut self, colors: &[C; Lights::COUNT]) {
        self.colors.clone_from(colors);
    }

    /// Flush the buffer's contents to lights, blending according to alpha level (if applicable).
    pub fn flush(&self) {
        let lights = MainFramebuffer::get();

        let colors = self.get_rgb_buffer_for_flush(lights);

        lights.set_all_colors(&colors);
    }

    /// Flush, and immediately clear the Framebuffer to avoid cumulative drawing when re-using it.
    pub fn flush_and_clear(&mut self) {
        self.flush();
        self.clear();
    }

    /// Clear the framebuffer from all data to revert all lights to [`OFF`][crate::color::OFF] for future re-use.
    /// Note : This function does not flush the buffer to the lights, but only clears the buffer data.
    pub fn clear(&mut self) {
        self.colors = [C::OFF; Lights::COUNT];
    }

    /// Blend `self` with `rhs`, using the same factor (`rhs_percent`) for each pixel.
    pub fn blend_scalar(&self, rhs: &Self, rhs_percent: f32) -> Self {
        let mut colors = [C::OFF; Lights::COUNT];
        let mut i = 0;
        while i < Lights::COUNT {
            colors[i] = self.colors[i].blend(rhs.colors[i], rhs_percent);
            i += 1;
        }
        Framebuffer { colors }
    }

    /// Dim `self` to `dim_percent`.
    /// Passing a `dim_percent` value of `0.0` returns `self`, whereas a value of `1.0` returns a
    /// blank/off buffer.
    pub fn dim_scalar(&self, dim_percent: f32) -> Self {
        let mut colors = [C::OFF; Lights::COUNT];
        let mut i = 0;
        while i < Lights::COUNT {
            colors[i] = self.colors[i].blend(C::OFF, dim_percent);
            i += 1;
        }
        Framebuffer { colors }
    }

    /// Blend `self` with `rhs`, using the corresponding item in `rhs_arr` to blend each pixel
    /// independently.
    ///
    /// If you want to blend two [`Framebuffer<RGBA>`] together, see
    /// [`alpha_blend`][Framebuffer<RGBA>::alpha_blend]
    pub fn blend(&self, rhs: &Self, rhs_arr: [f32; Lights::COUNT]) -> Self {
        let mut colors = [C::OFF; Lights::COUNT];
        let mut i = 0;
        while i < Lights::COUNT {
            colors[i] = self.colors[i].blend(rhs.colors[i], rhs_arr[i]);
            i += 1;
        }
        Framebuffer { colors }
    }

    /// Dim `self` to `dim_percent`, using the corresponding item in `dim_arr` for each pixel.
    /// Passing a `dim_percent` value of `0.0` returns `self`, whereas a value of `1.0` returns a
    /// blank/off buffer.
    pub fn dim(&self, dim_arr: [f32; Lights::COUNT]) -> Self {
        let mut colors = [C::OFF; Lights::COUNT];
        let mut i = 0;
        while i < Lights::COUNT {
            colors[i] = self.colors[i].blend(C::OFF, dim_arr[i]);
            i += 1;
        }
        Framebuffer { colors }
    }

    /// Apply `f` to each pixel in this buffer.
    pub fn map<V: Color, F: FnMut(C) -> V>(&self, f: F) -> Framebuffer<V> {
        Framebuffer {
            colors: self.colors.map(f),
        }
    }

    /// Blends the current light colors with this Framebuffer according to `self`'s alpha level.
    fn get_rgb_buffer_for_flush(&self, lights: &MainFramebuffer) -> [RGB; Lights::COUNT] {
        lights
            .get_currently_set()
            .blend(
                &self.map(|c| *rgb::bytemuck::from_bytes::<RGB>(&rgb::bytemuck::bytes_of(&c)[..2])),
                self.colors.map(|c| {
                    // idx 3 will be `Some` for RGBA, and `None` for RGB.
                    f32::from(*rgb::bytemuck::bytes_of(&c).get(3).unwrap_or(&255)) / 255.0
                }),
            )
            .colors
    }
}

impl Framebuffer<RGB> {
    /// Flush a framebuffer by only updating a given set of lights instead of the
    /// whole board.
    pub fn flush_with_mask(&self, mask: Lights) {
        MainFramebuffer::get().set_colors_on_selection(mask, &self.colors);
    }
    ///
    /// Flush all lights that are any value besides `color::OFF`. All other lights
    /// will be left unchanged.
    ///
    // TODO: this should probably be removed when we get a framebuffer that has
    // alpha channel support which flushes according to alpha level. This
    // work around does not work for painting "black" (aka off).
    pub fn flush_on_colors(&self) {
        let on_lights = Lights::from_indices(
            self.colors
                .iter()
                .enumerate()
                .filter(|&(_, c)| *c != crate::color::OFF)
                .map(|(idx, _)| idx),
        );
        self.flush_with_mask(on_lights);
    }

    /// Flush a framebuffer by only updating a given set of lights instead of the
    /// whole board, then immediately clear its underlying data for non-cumulative re-use.
    pub fn flush_with_mask_and_clear(&mut self, mask: Lights) {
        self.flush_with_mask(mask);
        self.clear();
    }

    /// Convert `self` to `Framebuffer<RGBA>`, using `a` as the alpha value for each pixel.
    pub fn with_alpha(self, a: u8) -> Framebuffer<RGBA> {
        Framebuffer {
            colors: self.colors.map(|c| c.with_alpha(a)),
        }
    }
}

impl Framebuffer<RGBA> {
    /// Flush the buffer's contents to lights, ignoring alpha level.
    pub fn flush_no_alpha(&self) {
        self.to_rgb().flush();
    }

    /// Convert `self` to `Framebuffer<RGB>`, ignoring alpha values.
    ///
    /// If you want to convert to RGB without ignoring alpha, use [`dim_to_rgb`].
    pub fn to_rgb(&self) -> Framebuffer<RGB> {
        Framebuffer {
            colors: self.colors.map(|c| c.rgb()),
        }
    }

    /// Convert `self` to `Framebuffer<RGB>`, dimming values according to alpha level.
    pub fn dim_to_rgb(&self) -> Framebuffer<RGB> {
        Framebuffer {
            colors: self.colors.map(|c| {
                c.rgb()
                    .map_colors(|comp| (f32::from(comp) * (f32::from(c.a) / 255.)) as u8)
            }),
        }
    }

    /// Blend `self` with `rhs`, using both buffers' alphas.
    pub fn alpha_blend(&self, rhs: &Self) -> Self {
        let mut out_buffer = [RGBA::OFF; Lights::COUNT];
        let mut i = 0;

        while i < Lights::COUNT {
            let lhs = self.colors[i].map(|c| f32::from(c) / 255.);
            let rhs = rhs.colors[i].map(|c| f32::from(c) / 255.);
            let alpha_factor = lhs.a / rhs.a;
            let alpha_factor2 = (1.0 - lhs.a) / rhs.a;

            out_buffer[i] = rgb::RGBA::<f32>::new(
                lhs.r * alpha_factor + rhs.r * alpha_factor2,
                lhs.g * alpha_factor + rhs.g * alpha_factor2,
                lhs.b * alpha_factor + rhs.b * alpha_factor2,
                1. - (1. - lhs.a) * (1. - rhs.a),
            )
            .map(|c| (c * 255.) as u8);

            i += 1;
        }

        Framebuffer { colors: out_buffer }
    }
}

macro_rules! impl_framebuffer_op {
    ($op:ident, $fn:ident) => {
        impl<C: Color + $op<C, Output = C>> $op<Framebuffer<C>> for Framebuffer<C> {
            type Output = Self;
            fn $fn(self, rhs: Framebuffer<C>) -> Self::Output {
                let mut colors = [C::OFF; Lights::COUNT];

                let mut i = 0;
                while i < Lights::COUNT {
                    colors[i] = self.colors[i].$fn(rhs.colors[i]);
                    i += 1;
                }

                Framebuffer { colors }
            }
        }
    };
}

impl_framebuffer_op!(Add, add);
impl_framebuffer_op!(Mul, mul);
impl_framebuffer_op!(Sub, sub);
impl_framebuffer_op!(Div, div);
