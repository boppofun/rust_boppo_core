use crate::{Buttons, Lights, MainFramebuffer, color};

/// On "off-screen" buffer storing a color for each light on the tablet.
///
/// These colors can be modifed with a simlar API as `[Buttons]` but will
/// not have any affect until `[flush]` is called.
#[derive(Clone)]
pub struct Framebuffer {
    /// A color for each Light on the tablet.
    /// The ordering is the same as [`Lights`].
    pub colors: [color::RGB; Lights::COUNT],
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Framebuffer {
    /// Create a new Framebuffer with all pixels set to OFF (Black).
    #[must_use]
    pub fn new() -> Self {
        Self {
            colors: [color::OFF; Lights::COUNT],
        }
    }

    /// Set every light in `selection` to `color`.
    pub fn set_color<B: Into<Lights>>(&mut self, selection: B, color: color::RGB) {
        let selection = selection.into();
        for lights_idx in selection.indices() {
            self.colors[lights_idx] = color;
        }
    }

    /// Set every button in `buttons` to the corresponding color in `colors`, up until either all
    /// buttons selected in `buttons` have been used, or all colors in `colors` have been used.
    pub fn set_buttons_colors(
        &mut self,
        buttons: Buttons,
        colors: impl IntoIterator<Item = crate::color::RGB>,
    ) {
        for (button, color) in buttons.buttons().zip(colors) {
            self.set_color(button, color);
        }
    }

    /// Set every light to the corresponding color in `colors`.
    pub fn set_all_colors(&mut self, colors: &[color::RGB; Lights::COUNT]) {
        self.colors = *colors;
    }

    // TODO(Ben Harris): Make the light flush behaviour here more explicit.
    /// Flush the buffer's contents to lights.
    pub fn flush(&self) {
        MainFramebuffer::get().set_all_colors(&self.colors);
    }

    /// Flush a framebuffer by only updating a given set of lights instead of the
    /// whole board.
    pub fn flush_with_mask(&self, mask: Lights) {
        MainFramebuffer::get().set_colors_on_selection(mask, &self.colors);
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
                .map(|c| c != color::OFF)
                .iter()
                .enumerate()
                .filter(|&(_, on)| *on)
                .map(|(idx, _)| idx),
        );
        self.flush_with_mask(on_lights);
    }

    /// Clear the framebuffer from all data to revert all lights to `color::OFF` for future re-use.
    /// Note : This function does not flush the buffer to the lights, but only clears the buffer data.
    pub fn clear(&mut self) {
        self.set_color(Lights::all(), color::OFF);
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
