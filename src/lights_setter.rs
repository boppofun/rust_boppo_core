use crate::Framebuffer;
use crate::Lights;

use crate::color::RGB;
use crate::hal;
use crate::hal::SetAndFlushLights;
use std::sync::Arc;
use std::sync::Mutex;

/// Low level control over setting the color of lights.
///
/// By default lights changes are immediately flushed to the hardware when
/// [`set_color`][`LightsSetter::set_color`] or similar functions are called
/// throughout this crate. Automatic flushing can be disabled via
/// [`pause_auto_flush`][`LightsSetter::pause_auto_flush`] and
/// [`set_auto_flush`][`LightsSetter::set_auto_flush`].
///
/// Get a reference to the global instance using [`LightsSetter::get()`].
#[derive(Clone)]
pub struct LightsSetter {
    inner: Arc<Mutex<LightsSettingInner>>,
}

impl LightsSetter {
    /// Returns a reference to the global [`LightsSetter`].
    ///
    /// # Panics
    ///
    /// This function only panics if this library was not initialized properly.
    pub fn get() -> &'static LightsSetter {
        crate::hal::LIGHTS.get().unwrap()
    }

    #[doc(hidden)]
    pub fn new(hal: SetAndFlushLights) -> LightsSetter {
        let inner = LightsSettingInner {
            hal,
            buffer: Framebuffer::new(),
            auto_flush: true,
        };
        let inner = Arc::new(Mutex::new(inner));
        LightsSetter { inner }
    }

    /// Sets the color of the specified lights to the given color.
    ///
    /// ## See also
    ///
    /// * [`Button.set_color`][crate::Button::set_color]
    /// * [`Buttons.set_color`][crate::Buttons::set_color]
    /// * [`Lights.set_color`][crate::Lights::set_color]
    #[expect(
        clippy::missing_panics_doc,
        reason = "Mutex is internal and should not poisen"
    )]
    pub fn set_color(&self, selection: Lights, color: RGB) {
        let mut inner = self.inner.lock().unwrap();
        inner.buffer.set_color(selection, color);
        if inner.auto_flush {
            inner.flush();
        }
    }

    /// Sets the color of all the lights to the given colors.
    ///
    /// ## See also
    ///
    /// * [`Lights.set_colors`][crate::Lights::set_color]
    #[expect(
        clippy::missing_panics_doc,
        reason = "Mutex is internal and should not poisen"
    )]
    pub fn set_all_colors(&self, colors: &[RGB; Lights::COUNT]) {
        let mut inner = self.inner.lock().unwrap();
        inner.buffer.set_all_colors(colors);

        if inner.auto_flush {
            inner.flush();
        }
    }

    /// Sets colors from the provided slice, ignoring any color that is not part of `selection`.
    #[expect(
        clippy::missing_panics_doc,
        reason = "Mutex is internal and should not poisen"
    )]
    pub fn set_colors_on_selection(&self, selection: Lights, colors: &[RGB; Lights::COUNT]) {
        let mut inner = self.inner.lock().unwrap();
        // TODO add this function to framebuffer
        for idx in selection.indices() {
            inner.buffer.colors[idx] = colors[idx];
        }
        if inner.auto_flush {
            inner.flush();
        }
    }

    /// Set each light in `selection` to the corresponding color in `colors`, where the first light
    /// in `selection` is `colors[0]`, the second is `colors[1]`, up to `selection.len().min(colors.len())`.
    ///
    /// NOTE: If `selection.len() != colors.len()`, extra lights will be ignored, or extra colors
    /// will be ignored, depending on which contains fewer elements.
    #[expect(
        clippy::missing_panics_doc,
        reason = "Mutex is internal and should not poisen"
    )]
    pub fn set_colors_on(&self, selection: Lights, colors: &[RGB]) {
        let mut inner = self.inner.lock().unwrap();

        // TODO add this function to framebuffer
        for (light_idx, color) in selection.indices().zip(colors.iter()) {
            inner.buffer.colors[light_idx] = *color;
        }

        if inner.auto_flush {
            inner.flush();
        }
    }

    /// Flush all pending color changes to the hardware for visual change.
    #[expect(
        clippy::missing_panics_doc,
        reason = "Mutex is internal and should not poisen"
    )]
    pub fn flush(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.flush();
    }

    /// Enable or disable auto flushing of color changes to the hardware.
    #[expect(
        clippy::missing_panics_doc,
        reason = "Mutex is internal and should not poisen"
    )]
    pub fn set_auto_flush(&self, v: bool) {
        let mut inner = self.inner.lock().unwrap();
        inner.auto_flush = v;
    }

    /// Return the currently set colors.
    ///
    /// These colors may or may not have been flushed to the hardware.
    #[expect(
        clippy::missing_panics_doc,
        reason = "Mutex is internal and should not poisen"
    )]
    pub fn get_currently_set(&self) -> Framebuffer {
        let inner = self.inner.lock().unwrap();
        inner.buffer.clone()
    }

    /// Turn off auto flushing until the returned Guard is dropped.
    /// When the guard is dropped all pending light changes are flushed.
    /// If autoflush was on when this function was called it will be reenabled unconditionally.
    #[expect(
        clippy::missing_panics_doc,
        reason = "Mutex is internal and should not poisen"
    )]
    #[must_use]
    pub fn pause_auto_flush(&self) -> PauseAutoFlushGuard {
        let mut inner = self.inner.lock().unwrap();
        let should_reenable = inner.auto_flush;
        inner.auto_flush = false;
        PauseAutoFlushGuard {
            should_reenable,
            lights_setter: self.clone(),
        }
    }
}

impl std::fmt::Debug for LightsSetter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lights").finish()
    }
}

struct LightsSettingInner {
    /// Do not call flush methods on this framebuffer
    pub buffer: Framebuffer,
    pub auto_flush: bool,
    pub hal: hal::SetAndFlushLights,
}

impl LightsSettingInner {
    pub fn flush(&mut self) {
        (self.hal)(&self.buffer.colors);
    }
}

/// Pauses auto flushing until the guard is dropped.
pub struct PauseAutoFlushGuard {
    lights_setter: LightsSetter,
    should_reenable: bool,
}

impl Drop for PauseAutoFlushGuard {
    fn drop(&mut self) {
        if self.should_reenable {
            self.lights_setter.set_auto_flush(true);
        }
        self.lights_setter.flush();
    }
}
