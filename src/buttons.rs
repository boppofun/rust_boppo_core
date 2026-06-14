use rand::prelude::IteratorRandom;
use serde::Deserialize;
use serde::de::Error;

use crate::internal::BUTTON_COUNTS;
use crate::lights::LightDir;
use crate::{Column, Lights, Row};

use super::button::Button;
use std::convert::TryInto;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
/// A selection of multiple [`Button`s][`Button`].
pub struct Buttons {
    // bitset where lowest order bit represents the first button
    bits: u16,
}

impl Buttons {
    /// Returns a selection containing all indices marked true in `arr`.
    ///
    /// # Panics
    ///
    /// This function will panic if `arr.len() > Button::COUNT`
    #[must_use]
    pub fn from_slice(arr: &[bool]) -> Buttons {
        assert!(arr.len() <= Button::COUNT);
        Self::from_indices(
            arr.iter()
                .enumerate()
                .filter(|&(_, p)| *p)
                .map(|(idx, _)| idx),
        )
    }

    #[must_use]
    /// Returns a new [`Buttons`] at index `button_index`.
    ///
    /// # Panics
    ///
    /// This function will panic if `button_index >= Button::COUNT`.
    pub const fn from_index(button_index: usize) -> Buttons {
        Self::from_bitset(1 << button_index)
    }

    /// Returns a new [`Buttons`] constructed from button indices contained in `indexes`.
    ///
    /// # Panics
    ///
    /// This function will panic if `indexes` contains an `index >= Button::COUNT`.
    pub fn from_indices<I>(indexes: I) -> Buttons
    where
        I: IntoIterator<Item = usize>,
    {
        let mut bits: u16 = 0;
        for index in indexes {
            assert!(index < Button::COUNT);
            bits |= 1 << index;
        }
        Buttons { bits }
    }

    /// Returns a new [`Buttons`] with `bits`. Each bit in `bits` represents whether an individual
    /// button has been selected. The 0th bit selects [`Button::B0`].
    ///
    /// # Panics
    ///
    /// This function will panic if `bits >> Button::COUNT != 0`.
    #[must_use]
    pub const fn from_bitset(bits: u16) -> Buttons {
        assert!(bits >> Button::COUNT == 0);
        Buttons { bits }
    }

    /// Returns a [`Buttons`] with every button selected.
    #[must_use]
    pub const fn all() -> Buttons {
        Buttons::from_bitset((1 << Button::COUNT) - 1)
    }

    /// Returns a [`Buttons`] with no button selected.
    #[must_use]
    pub const fn none() -> Buttons {
        Self::from_bitset(0)
    }

    /// Returns a [`Buttons`] with only the buttons in `row` selected.
    #[must_use]
    pub const fn row(row: Row) -> Buttons {
        match row {
            Row::Top => Self::from_bitset(0b0001_1111),
            Row::Bottom => Self::from_bitset(0b0011_1110_0000),
        }
    }

    /// Returns the two buttons in `col` (one per row).
    ///
    /// # Example
    ///
    /// ```
    /// use boppo_core::{Buttons, Column};
    /// let mut indices = Buttons::column(Column::C0).indices();
    /// assert_eq!(indices.next(), Some(0));
    /// assert_eq!(indices.next(), Some(5));
    /// ```
    #[must_use]
    pub fn column(col: Column) -> Buttons {
        Self::from_indices([col.index(), 5 + col.index()])
    }

    /// Return all buttons that are currently held down.
    ///
    /// See also [`ButtonEvents`][crate::ButtonEvents] which is an alternative way to
    /// receive button events.
    #[expect(
        clippy::missing_panics_doc,
        reason = "This only panics if the library hasn't been initialised properly."
    )]
    pub fn currently_pressed() -> Buttons {
        BUTTON_COUNTS.get().unwrap().borrow().currently_pressed()
    }

    /// Returns the number of buttons in this selection.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.bits.count_ones()
    }

    /// Returns `true` if no buttons are selected.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if this selection contains `button`.
    #[must_use]
    pub const fn contains(&self, button: Button) -> bool {
        self.is_superset(button.to_buttons())
    }

    /// Returns `true` if `other` selects only buttons that `self` selects.
    ///
    /// See also [`Buttons::is_subset`]
    #[must_use]
    pub const fn is_superset(&self, other: Self) -> bool {
        (self.bits | other.bits) == self.bits
    }

    /// Returns `true` if `self` selects only buttons that `other` selects.
    ///
    /// See also [`Buttons::is_superset`]
    #[must_use]
    pub const fn is_subset(&self, other: Self) -> bool {
        other.is_superset(*self)
    }

    /// Returns a double-ended iterator over the indices of every [`Button`] in this selection.
    #[must_use]
    pub const fn indices(&self) -> impl DoubleEndedIterator<Item = usize> + use<> {
        Indices { bits: self.bits }
    }

    /// Returns a double-ended iterator over every [`Button`] in this selection.
    #[must_use]
    pub fn buttons(&self) -> impl DoubleEndedIterator<Item = Button> + use<> {
        self.indices().map(|index| Button::from_index(index))
    }

    /// Returns the inversion of the current selection.
    #[must_use]
    pub const fn invert(&self) -> Buttons {
        Buttons {
            bits: (!self.bits) & Buttons::all().bits,
        }
    }

    /// Randomly choose `n` of the buttons that are active in `self`.
    /// If `n == 1`, it's better to use [`choose_one_randomly`][Buttons::choose_one_randomly].
    #[must_use]
    pub fn choose_n_randomly(&self, n: usize) -> Buttons {
        let mut chosen = [11; Button::COUNT];
        let num_chosen = self
            .indices()
            .choose_multiple_fill(&mut rand::rng(), &mut chosen[0..n]);
        Buttons::from_indices(chosen[0..num_chosen].iter().copied())
    }

    /// Randomly choose one [`Button`] from those active in `self`.
    #[must_use]
    pub fn choose_one_randomly(&self) -> Button {
        // The compiler is smart enough to optimise even this nasty chain, making this wayy better than
        // `choose_n_randomly` for this common case.
        Button::from_index(self.choose_n_randomly(1).as_bitset().trailing_zeros() as usize)
    }

    /// Return the Buttons as if the device was rotated 180 degrees.
    /// Button 1 becomes 10, 2 becomes 9... 5 becomes 6 and vice versa.
    #[must_use]
    pub const fn rotate_180(&self) -> Buttons {
        Buttons::from_bitset(self.bits.reverse_bits() >> 6)
    }

    /// A compact representation. The least significant bit represents if button 0 is pressed...
    #[must_use]
    pub const fn as_bitset(&self) -> u16 {
        self.bits
    }

    /// Set [`self's`][`crate::Button`] lights to [`color`][`crate::color`].
    pub fn set_color(self, color: crate::color::RGB) {
        crate::MainFramebuffer::get().set_color(self.into(), color);
    }

    /// Set [`self's`][`crate::Button`] lights to [`color::OFF`][`crate::color::OFF`].
    /// Shorthand for [`self.set_color(color::OFF)`][crate::Button::set_color]
    pub fn set_off(self) {
        self.set_color(crate::color::OFF);
    }

    /// Set the lowest button in this selection to `colors[0]` and second lowest
    /// to `colors[1]` and so on.
    /// If there are more colors than buttons the extra colors are ignored. If
    /// there are less colors than buttons, the color of the extra buttons will
    /// remain unchanged.
    pub fn set_colors(self, colors: impl IntoIterator<Item = crate::color::RGB>) {
        for (button, color) in self.buttons().zip(colors) {
            button.set_color(color);
        }
    }

    /// Returns a [`Lights`] containing every light for every [`Button`] in this selection.
    #[must_use]
    pub const fn lights(self) -> Lights {
        let input = self.as_bitset() as u64;
        let mut result: u64 = 0;
        let mut i = 0;
        while i < 10 {
            // Check if bit i is set in input
            if (input & (1 << i)) != 0 {
                // Set 4 bits in the output starting at position 4*i
                result |= 0b1111u64 << (4 * i);
            }
            i += 1;
        }
        Lights::from_bitset(result)
    }

    /// Returns a [`Lights`] containing only the light at `dir` for every [`Button`] in this selection.
    #[must_use]
    pub const fn lights_on(self, dir: LightDir) -> Lights {
        let lights: Lights = self.lights();
        lights.only(dir)
    }

    /// Returns an iterator over every [`Button`] in this selection.
    #[must_use]
    pub fn iter(&self) -> ButtonsIter {
        ButtonsIter { bits: self.bits }
    }
}

impl IntoIterator for &Buttons {
    type IntoIter = ButtonsIter;
    type Item = Button;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl std::ops::BitAnd for Buttons {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Buttons {
            bits: self.bits & rhs.bits,
        }
    }
}

impl std::ops::BitAndAssign for Buttons {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl std::ops::BitOr for Buttons {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Buttons {
            bits: self.bits | rhs.bits,
        }
    }
}

impl std::ops::BitOrAssign for Buttons {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl std::ops::BitXor for Buttons {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Buttons {
            bits: self.bits ^ rhs.bits,
        }
    }
}

impl std::ops::BitXorAssign for Buttons {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bits ^= rhs.bits;
    }
}

impl std::ops::Not for Buttons {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.invert()
    }
}

impl std::iter::IntoIterator for Buttons {
    type Item = Button;
    type IntoIter = ButtonsIter;

    /// Iterates in order of index.
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl From<Button> for Buttons {
    fn from(button: Button) -> Self {
        Buttons::from_index(button.index())
    }
}

impl From<&[Button]> for Buttons {
    fn from(buttons: &[Button]) -> Self {
        buttons.iter().copied().collect()
    }
}

impl FromIterator<Button> for Buttons {
    fn from_iter<T: IntoIterator<Item = Button>>(iter: T) -> Self {
        let mut res = Buttons::none();
        for button in iter {
            res |= button.into();
        }
        res
    }
}

/// An iterator over the [`Button`]s in a [`Buttons`] selection, in index order.
pub struct ButtonsIter {
    bits: u16,
}

impl Iterator for ButtonsIter {
    type Item = Button;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let next = self.bits.trailing_zeros() as usize;
            self.bits &= self.bits - 1;
            Some(Button::from_index(next))
        }
    }
}

impl DoubleEndedIterator for ButtonsIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let next = self.bits.ilog2() as usize;
            self.bits ^= 1u16 << next;
            Some(Button::from_index(next))
        }
    }
}

struct Indices {
    bits: u16,
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
            self.bits ^= 1u16 << next;
            Some(next as usize)
        }
    }
}

impl<'de> Deserialize<'de> for Buttons {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        Ok(match value {
            serde_json::Value::Number(number) => number
                .as_u64()
                .ok_or(D::Error::custom("number too large"))
                .and_then(|idx| {
                    usize::try_from(idx).map_err(|_| D::Error::custom("number too large"))
                })
                .and_then(|idx| {
                    if idx < Button::COUNT {
                        Ok(idx)
                    } else {
                        Err(D::Error::custom("number too large"))
                    }
                })
                .map(Buttons::from_index)?,
            serde_json::Value::String(s) => {
                if s == "All" {
                    Buttons::all()
                } else {
                    return Err(D::Error::custom("Unexpected JSON string value for Buttons"));
                }
            }
            serde_json::Value::Array(values) => {
                let numbers: Result<Vec<usize>, _> = values
                    .iter()
                    .map(|v| {
                        v.as_number()
                            .and_then(serde_json::Number::as_u64)
                            .ok_or(D::Error::custom("Buttons array has non-number"))
                            .and_then(|n| {
                                usize::try_from(n).map_err(|_| D::Error::custom("number too large"))
                            })
                            .and_then(|idx| {
                                if idx < Button::COUNT {
                                    Ok(idx)
                                } else {
                                    Err(D::Error::custom("number too large"))
                                }
                            })
                    })
                    .collect();

                Buttons::from_indices(numbers?)
            }
            _ => return Err(D::Error::custom("Unexpected JSON type for Buttons")),
        })
    }
}

#[cfg(test)]
#[path = "./tests/buttons_test.rs"]
mod test;
