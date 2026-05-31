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

/// [`Framebuffer`] using [`RGBA`] as its pixel type.
pub type FramebufferRGBA = Framebuffer<RGBA>;

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
        let lights = MainFramebuffer::get();

        let colors = self.get_rgb_buffer_for_flush(lights);

        lights.set_all_colors(&colors);
    }

    /// Clear the framebuffer from all data to revert all lights to [`OFF`][crate::color::OFF] for future re-use.
    /// Note : This function does not flush the buffer to the lights, but only clears the buffer data.
    pub fn clear(&mut self) {
        self.colors = [C::OFF; Lights::COUNT];
    }

    /// Blend `self` with `rhs`, using `blend_fn`.
    ///
    /// ```rust
    /// # use boppo_core::{Framebuffer, color, Lights};
    /// let red_fb = Framebuffer {
    ///     colors: [color::RED; Lights::COUNT]
    /// };
    /// let blue_fb = Framebuffer {
    ///     colors: [color::BLUE; Lights::COUNT]
    /// };
    ///
    /// // All purple
    /// let purple_fb = red_fb.blend(&blue_fb, |c1, c2, _| c1.weighted_average(c2, 0.5));
    ///
    /// // Fades from red to blue across Button::B0-Button::B4 then Button::B5-Button::B9
    /// let smooth_fb = red_fb.blend(&blue_fb, |c1, c2, i| c1.weighted_average(c2, (i / 5) as f32 / Lights::COUNT as f32));
    /// ```
    ///
    /// If you want to blend two [`Framebuffer<RGBA>`] together, see
    /// [`alpha_blend`][Framebuffer<RGBA>::alpha_blend]
    pub fn blend<W: Fn(C, C, usize) -> C>(&self, rhs: &Self, blend_fn: W) -> Self {
        let mut colors = [C::OFF; Lights::COUNT];
        let mut i = 0;
        while i < Lights::COUNT {
            colors[i] = blend_fn(self.colors[i], rhs.colors[i], i);
            i += 1;
        }
        Framebuffer { colors }
    }

    /// Dim `self` to `dim_percent`, using the corresponding item in `dim_arr` for each pixel.
    /// Passing a `dim_percent` value of `0.0` returns `self`, whereas a value of `1.0` returns a
    /// blank/off buffer.
    pub fn dim<W: Fn(C, usize) -> C>(&self, dim_fn: W) -> Self {
        let mut colors = [C::OFF; Lights::COUNT];
        let mut i = 0;
        while i < Lights::COUNT {
            colors[i] = dim_fn(self.colors[i], i);
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
        let alphas = self.colors.map(|c| {
            // idx 3 will be `Some` for RGBA, and `None` for RGB.
            f32::from(*rgb::bytemuck::bytes_of(&c).get(3).unwrap_or(&255)) / 255.0
        });
        lights
            .get_currently_set()
            .blend(
                &self.map(|c| *rgb::bytemuck::from_bytes::<RGB>(&rgb::bytemuck::bytes_of(&c)[..3])),
                |c1, c2, i| c1.weighted_average(c2, alphas[i]),
            )
            .colors
    }
}

impl Framebuffer<RGB> {
    /// Flush this buffer to lights, blending with the current state according to `blend_fn`.
    /// `blend_fn` is passed the light index, the current light color, and this buffer's light
    /// color, in that order.
    pub fn flush_blend<W: Fn(usize, RGB, RGB) -> RGB>(&self, blend_fn: W) {
        let lights = MainFramebuffer::get();

        let current = lights.get_currently_set();

        let _guard = lights.pause_auto_flush();
        for i in 0..Lights::COUNT {
            lights.set_color(
                Lights::from_index(i),
                blend_fn(i, current.colors[i], self.colors[i]),
            );
        }
    }

    /// Convert `self` to `Framebuffer<RGBA>`, using `a` as the alpha value for each pixel.
    pub fn with_alpha(self, a: u8) -> Framebuffer<RGBA> {
        Framebuffer {
            colors: self.colors.map(|c| c.with_alpha(a)),
        }
    }

    /// Blends this framebuffer with the current [`MainFramebuffer`], and writes the result back to
    /// this buffer.
    pub fn blend_with_current(&mut self) {
        let current = MainFramebuffer::get().get_currently_set();
        *self = self.blend(&current, |c1, c2, _| c1.blend(c2));
    }
}

impl Framebuffer<RGBA> {
    /// Flush this buffer to lights, blending with the current state according to `blend_fn`.
    /// `blend_fn` is passed the light index, the current light color, and this buffer's light
    /// color, in that order.
    pub fn flush_blend<W: Fn(usize, RGB, RGBA) -> RGB>(&self, blend_fn: W) {
        let lights = MainFramebuffer::get();

        let current = lights.get_currently_set();

        let _guard = lights.pause_auto_flush();
        for i in 0..Lights::COUNT {
            lights.set_color(
                Lights::from_index(i),
                blend_fn(i, current.colors[i], self.colors[i]),
            );
        }
    }

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

    /// Blends this framebuffer with the current [`MainFramebuffer`], and writes the result back to
    /// this buffer.
    pub fn blend_with_current(&mut self) {
        let current = MainFramebuffer::get().get_currently_set().with_alpha(255);
        *self = self.blend(&current, |c1, c2, _| c1.blend(c2));
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
