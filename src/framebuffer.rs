use std::ops::{Add, Div, Mul, Sub};

use crate::{Buttons, Lights, MainFramebuffer, color::ColorExt};

/// On "off-screen" buffer storing a color for each light on the tablet.
///
/// These colors can be modifed with a simlar API as `[Buttons]` but will
/// not have any affect until `[flush]` is called.
#[derive(Clone, PartialEq)]
pub struct Framebuffer<C = crate::color::RGB> {
    /// A color for each Light on the tablet.
    /// The ordering is the same as [`Lights`].
    pub colors: [C; Lights::COUNT],
}

impl<C: Default + Copy> Default for Framebuffer<C> {
    fn default() -> Self {
        Framebuffer {
            colors: [C::default(); Lights::COUNT],
        }
    }
}

impl<C: ColorExt> Framebuffer<C> {
    /// Create a new Framebuffer with all pixels set to OFF (Black).
    #[must_use]
    pub fn new() -> Self {
        Self {
            colors: [C::OFF; Lights::COUNT],
        }
    }

    /// Set every light in `selection` to `color`.
    pub fn set_color<B: Into<Lights>>(&mut self, selection: B, color: C) {
        let selection = selection.into();
        for lights_idx in selection.indices() {
            self.colors[lights_idx] = color.clone();
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
}

impl<C: ColorExt<RGB8 = crate::color::RGB>> Framebuffer<C> {
    // TODO(Ben Harris): Make the light flush behaviour here more explicit.
    /// Flush the buffer's contents to lights.
    pub fn flush(&self) {
        MainFramebuffer::get().set_all_colors(&self.colors.clone().map(|c| c.to_rgb_u8()));
    }

    /// Flush a framebuffer by only updating a given set of lights instead of the
    /// whole board.
    pub fn flush_with_mask(&self, mask: Lights) {
        MainFramebuffer::get()
            .set_colors_on_selection(mask, &self.colors.clone().map(|c| c.to_rgb_u8()));
    }

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
                .filter(|&(_, c)| *c != C::OFF)
                .map(|(idx, _)| idx),
        );
        self.flush_with_mask(on_lights);
    }

    /// Clear the framebuffer from all data to revert all lights to `color::OFF` for future re-use.
    /// Note : This function does not flush the buffer to the lights, but only clears the buffer data.
    pub fn clear(&mut self) {
        self.set_color(Lights::all(), C::OFF);
    }

    /// Flush, and immediately clear the Framebuffer to avoid cumulative drawing when re-using it.
    pub fn flush_and_clear(&mut self) {
        self.flush();
        self.clear();
    }

    /// Flush a framebuffer by only updating a given set of lights instead of the
    /// whole board, then immediately clear its underlying data for non-cumulative re-use.
    pub fn flush_with_mask_and_clear(&mut self, mask: Lights) {
        self.flush_with_mask(mask);
        self.clear();
    }
}

impl<C: ColorExt + Copy> ColorExt for Framebuffer<C>
where
    C::RGB8: Copy,
{
    type Percent = Framebuffer<C::Percent>;
    type RGB8 = Framebuffer<C::RGB8>;

    const OFF: Self = Framebuffer {
        colors: [C::OFF; Lights::COUNT],
    };

    fn blend(self, rhs: Self, rhs_percent: Self::Percent) -> Self {
        let mut i = 0;
        let mut colors = [C::default(); Lights::COUNT];
        while i < Lights::COUNT {
            colors[i] = self.colors[i].blend(rhs.colors[i], rhs_percent.colors[i].clone());
            i += 1;
        }
        Framebuffer { colors }
    }

    fn dim_to(self, percent: Self::Percent) -> Self {
        self.blend(Self::default(), percent)
    }

    fn luminance(self) -> Self::Percent {
        Framebuffer {
            colors: self.colors.map(ColorExt::luminance),
        }
    }

    fn to_rgb_u8(&self) -> Self::RGB8 {
        Framebuffer {
            colors: self.colors.map(|c| c.to_rgb_u8()),
        }
    }
}

macro_rules! impl_framebuffer_op {
    ($op:ident, $fn:ident) => {
        impl<C: Clone + $op<C, Output = C>> $op<Framebuffer<C>> for Framebuffer<C> {
            type Output = Self;
            fn $fn(self, rhs: Framebuffer<C>) -> Self::Output {
                let colors: [C; 40];
                // Unsafe required here as we don't require `ColorExt` here, so `C` has no known default
                // value.
                // SAFETY: All array items are initialized before `assume_init` is called on any item.
                unsafe {
                    let mut colors_uninit =
                        [const { std::mem::MaybeUninit::uninit() }; Lights::COUNT];
                    let mut i = 0;
                    while i < Lights::COUNT {
                        colors_uninit[i].write(self.colors[i].clone().$fn(rhs.colors[i].clone()));
                        i += 1;
                    }
                    colors = colors_uninit.map(|c| c.assume_init());
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
