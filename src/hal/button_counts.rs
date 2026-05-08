use crate::{Button, Buttons};

/// Counts button presses with wrapping. Used to detect press changes.
#[derive(Debug, Copy, Clone, Default)]
pub struct ButtonCounts {
    /// an odd number count means is pressed, even means is released
    /// u16 is small enough that we need to consider wrapping
    counts: [u16; Button::COUNT],
}

impl ButtonCounts {
    /// Increment the count for `button`
    pub fn update_for_event(&mut self, button: Button, is_press: bool) {
        let count = &mut self.counts[button.index()];
        *count = count.wrapping_add(1);
        // ensure that presses are odds and releases are even
        let count_is_press = count_is_press(*count);
        if is_press != count_is_press {
            *count = count.wrapping_add(1);
        }
    }

    /// Return true if `button` is currently pressed according to the counts
    #[must_use]
    pub fn is_pressed(&self, button: Button) -> bool {
        count_is_press(self.counts[button.index()])
    }

    /// Return all `Buttons` currently pressed according to the counts
    #[must_use]
    pub fn currently_pressed(&self) -> Buttons {
        let is_presses = self.counts.map(count_is_press);
        Buttons::from_slice(&is_presses)
    }
}

/// Odd counts are press. Even counts are released.
fn count_is_press(count: u16) -> bool {
    count & 1 == 1
}
