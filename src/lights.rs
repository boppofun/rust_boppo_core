use rand::prelude::IteratorRandom;

use crate::{Buttons, MainFramebuffer};

use super::button::Button;
use std::convert::TryInto;

/// A selection of button lights on the tablet. There are 10 buttons with 4
/// lights each for a total of 40 lights.
///
/// # Ordering
///
/// The order is in [`Button'][crate::Button] order from Button::B0 to
/// Button::B9 where each button has 4 lights together in the following order:
///
/// * "Top"
/// * "Left"
/// * "Right"
/// * "Bottom"
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Lights {
    // bitset where lowest order bit represents the first button
    bits: u64,
}

impl Lights {
    /// The number of lights on Boppo. Each button has four lights.
    ///
    /// NOTE: that this is not the number of lights in this selection
    /// which is obtained by calling `self.len()`
    pub const COUNT: usize = super::Button::COUNT * 4;

    /// Returns a selection containing all indices marked true in `arr`.
    ///
    /// # Panics
    ///
    /// This function will panic if `arr.len() > Lights::COUNT`
    #[must_use]
    pub fn from_slice(arr: &[bool]) -> Lights {
        assert!(arr.len() <= Self::COUNT);
        Self::from_indices(
            arr.iter()
                .enumerate()
                .filter(|&(_, p)| *p)
                .map(|(idx, _)| idx),
        )
    }

    /// Returns a [`Lights`] with only the light at `light_index` selected.
    #[must_use]
    pub const fn from_index(light_index: usize) -> Lights {
        Self::from_bitset(1 << light_index)
    }

    /// Returns a new [`Lights`] constructed from light indices contained in `indexes`.
    ///
    /// # Panics
    ///
    /// This function will panic if `indexes` contains an `index >= Lights::COUNT`.
    pub fn from_indices<I>(indexes: I) -> Lights
    where
        I: IntoIterator<Item = usize>,
    {
        let mut bits: u64 = 0;
        for index in indexes {
            assert!(index < Self::COUNT);
            bits |= 1 << index;
        }
        Lights { bits }
    }

    /// Returns a new [`Lights`] with `bits`.
    ///
    /// Each bit in `bits` represents whether an individual light has been selected.
    /// The least significant bit selects `Button::B1.light_at(LightDir::Top)`.
    ///
    /// # Ordering
    ///
    /// See [Lights ordering](crate::Lights#ordering).
    ///
    /// # Panics
    ///
    /// This function will panic if `bits >> Lights::COUNT != 0`.
    #[must_use]
    pub const fn from_bitset(bits: u64) -> Lights {
        assert!(bits >> Self::COUNT == 0);
        Lights { bits }
    }

    /// All 4 lights of `button`
    #[must_use]
    pub const fn all_from_button(button: Button) -> Lights {
        Lights::from_bitset(0b1111 << (button.index() * 4))
    }

    /// Returns a [`Lights`] with every light selected
    #[must_use]
    pub const fn all() -> Lights {
        Lights {
            bits: (1 << Self::COUNT) - 1,
        }
    }

    /// Returns a [`Lights`] with no light selected
    #[must_use]
    pub const fn none() -> Lights {
        Self::from_bitset(0)
    }

    /// Returns the number of lights in this selection.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.bits.count_ones()
    }

    /// Returns `true` if no lights are selected.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if `other` selects only buttons that `self` selects.
    ///
    /// See also [`Lights::is_subset`]
    #[must_use]
    pub const fn is_superset(&self, other: Self) -> bool {
        (self.bits | other.bits) == self.bits
    }

    /// Returns `true` if `other` selects only buttons that `self` selects.
    ///
    /// See also [`Lights::is_superset`]
    #[must_use]
    pub const fn is_subset(&self, other: Self) -> bool {
        other.is_superset(*self)
    }

    /// Returns a double-ended iterator over the indices of every [`Button`] in this selection.
    #[must_use]
    pub const fn indices(&self) -> impl DoubleEndedIterator<Item = usize> + use<> {
        Indices { bits: self.bits }
    }

    /// Returns the inversion of the current selection.
    #[must_use]
    pub const fn invert(&self) -> Lights {
        Lights {
            bits: (!self.bits) & Lights::all().bits,
        }
    }

    // TODO(Ben Harris): We may as well have a matching `choose_one_randomly`. The only use of this
    // function only actually uses it to select 1 light.
    /// Randomly choose `n` of the lights that are active in `self`.
    #[must_use]
    pub fn choose_n_randomly(&self, n: usize) -> Lights {
        let mut chosen = [11; Self::COUNT];
        let num_chosen = self
            .indices()
            .choose_multiple_fill(&mut rand::rng(), &mut chosen[0..n]);
        Lights::from_indices(chosen[0..num_chosen].iter().copied())
    }

    /// Return the Lights as if the device was rotate 180 degrees.
    /// Light 0 becomes 39, 1 becomes 38...
    #[must_use]
    pub const fn rotate_180(&self) -> Lights {
        Lights::from_bitset(self.bits.reverse_bits() >> 24)
    }

    /// A compact representation. The least significant bit represents if button 1 is pressed...
    #[must_use]
    pub const fn as_bitset(&self) -> u64 {
        self.bits
    }

    /// Sets all lights in this selection to `color`
    pub fn set_color(self, color: crate::color::RGB) {
        MainFramebuffer::get().set_color(self, color);
    }

    /// Sets each light in this selection to the corresponding color in `colors`, up to exhaustion
    /// of either the selection or `colors`.
    pub fn set_colors(self, colors: &[crate::color::RGB]) {
        MainFramebuffer::get().set_colors_on(self, colors);
    }

    /// Returns a new [`Lights`] with only the lights in the `dir` direction from this selection.
    #[must_use]
    pub const fn only(self, dir: LightDir) -> Lights {
        let mask = dir.light_on_all_buttons().as_bitset();
        Lights::from_bitset(self.as_bitset() & mask)
    }
}

impl std::ops::BitAnd for Lights {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Lights {
            bits: self.bits & rhs.bits,
        }
    }
}

impl std::ops::BitAndAssign for Lights {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl std::ops::BitOr for Lights {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Lights {
            bits: self.bits | rhs.bits,
        }
    }
}

impl std::ops::BitOrAssign for Lights {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl std::ops::BitXor for Lights {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Lights {
            bits: self.bits ^ rhs.bits,
        }
    }
}

impl std::ops::BitXorAssign for Lights {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bits ^= rhs.bits;
    }
}

impl std::ops::Not for Lights {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.invert()
    }
}

impl std::iter::IntoIterator for Lights {
    type Item = (Button, LightDir);
    type IntoIter = Box<dyn Iterator<Item = (Button, LightDir)>>;

    /// Iterates in order of index.
    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.indices().map(|index| {
            let button_index = index / 4;
            let light_dir_index = index % 4;
            (
                Button::from_index(button_index),
                LightDir::from_index(light_dir_index).unwrap(),
            )
        }))
    }
}

impl From<Button> for Lights {
    fn from(button: Button) -> Self {
        Lights::all_from_button(button)
    }
}

impl From<Buttons> for Lights {
    fn from(buttons: Buttons) -> Self {
        buttons.lights()
    }
}

struct Indices {
    bits: u64,
}

impl Iterator for Indices {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let next = self.bits.trailing_zeros().try_into().unwrap();
            self.bits = self.bits & (self.bits - 1);
            Some(next)
        }
    }
}

impl DoubleEndedIterator for Indices {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let next = self.bits.ilog2();
            self.bits ^= 1 << u64::from(next);
            Some(next as usize)
        }
    }
}

/// The directions lights are positioned around the buttons.
///
/// When [`Row::Top`][crate::Row::Top] (e.g. [`Button::B0`]) is further away from the
/// user than [`Row::Bottom`][crate::Row::Bottom] (e.g. [`Button::B5`]), [`LightDir::Top`] is
/// also the furthest from the user and [`LightDir::Bottom`] is the closest.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
#[expect(
    missing_docs,
    reason = "Variants names are clear. Meaning of direction documented in enum documentation."
)]
pub enum LightDir {
    Top = 0,
    Left = 1,
    Right = 2,
    Bottom = 3,
}

impl LightDir {
    /// Returns all four [`LightDir`] variants, ordered clockwise starting at [`LightDir::Top`].
    #[must_use]
    pub const fn all_clockwise_from_top() -> &'static [Self] {
        &[
            LightDir::Top,
            LightDir::Right,
            LightDir::Bottom,
            LightDir::Left,
        ]
    }

    /// Returns the next [`LightDir`] clockwise from `self`.
    #[must_use]
    pub const fn clockwise(self) -> Self {
        match self {
            LightDir::Top => LightDir::Right,
            LightDir::Left => LightDir::Top,
            LightDir::Right => LightDir::Bottom,
            LightDir::Bottom => LightDir::Left,
        }
    }

    /// Returns the next [`LightDir`] counter-clockwise from `self`.
    #[must_use]
    pub const fn counter_clockwise(self) -> Self {
        match self {
            LightDir::Top => LightDir::Left,
            LightDir::Left => LightDir::Bottom,
            LightDir::Right => LightDir::Top,
            LightDir::Bottom => LightDir::Right,
        }
    }

    /// Returns [`Some`] if `idx < 4`, [`None`] otherwise. Converts `idx` to the corresponding
    /// [`LightDir`] variant.
    #[must_use]
    pub const fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(LightDir::Top),
            1 => Some(LightDir::Left),
            2 => Some(LightDir::Right),
            3 => Some(LightDir::Bottom),
            _ => None,
        }
    }

    /// Returns a [`Lights`] with every light that corresponds to this [`LightDir`] selected.
    ///
    /// # Example
    ///
    /// ```
    /// # use boppo_core::{Lights, LightDir};
    /// let all_lights = Lights::all();
    /// let top_lights = LightDir::Top.light_on_all_buttons();
    ///
    /// assert_eq!(top_lights, all_lights.only(LightDir::Top));
    /// ```
    #[must_use]
    pub const fn light_on_all_buttons(self) -> Lights {
        Lights::from_bitset(match self {
            LightDir::Top => 0x11111_11111,

            LightDir::Left => 0x22222_22222,

            LightDir::Right => 0x44444_44444,

            LightDir::Bottom => 0x88888_88888,
        })
    }
}

#[cfg(test)]
#[path = "./tests/lights_test.rs"]
mod test;
