use serde::de::Error;
use serde::{Deserialize, Deserializer};

use crate::Lights;
use crate::buttons::Buttons;
use crate::hal::BUTTON_COUNTS;
use crate::lights::LightDir;

/// One of the 10 top buttons.
///
/// Button 0 is the top left. 4 is the top right. 5 is the bottom left and 9 is
/// the bottom right. (English lexographical order).
///
/// Represented visually:
///
/// ```text
/// 0 1 2 3 4
/// 5 6 7 8 9
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Button {
    /// [`Row::Top`], [`Column::C0`]
    B0 = 0,
    /// [`Row::Top`], [`Column::C1`]
    B1 = 1,
    /// [`Row::Top`], [`Column::C2`]
    B2 = 2,
    /// [`Row::Top`], [`Column::C3`]
    B3 = 3,
    /// [`Row::Top`], [`Column::C4`]
    B4 = 4,
    /// [`Row::Bottom`], [`Column::C0`]
    B5 = 5,
    /// [`Row::Bottom`], [`Column::C1`]
    B6 = 6,
    /// [`Row::Bottom`], [`Column::C2`]
    B7 = 7,
    /// [`Row::Bottom`], [`Column::C3`]
    B8 = 8,
    /// [`Row::Bottom`], [`Column::C4`]
    B9 = 9,
}

impl Button {
    /// The number of light-up buttons on top of Boppo.
    pub const COUNT: usize = 10;

    #[track_caller]
    #[must_use]
    /// Converts an `index` to its corresponding [`Button`].
    ///
    /// # Panics
    ///
    /// This function will panic if `index >= Button::COUNT`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boppo_core::Button;
    /// let button = Button::from_index(6);
    /// assert_eq!(button, Button::B6);
    /// ```
    ///
    /// ```should_panic
    /// # use boppo_core::Button;
    /// // Panic!
    /// let button = Button::from_index(100);
    /// ```
    pub const fn from_index(index: usize) -> Button {
        use Button::{B0, B1, B2, B3, B4, B5, B6, B7, B8, B9};
        match index {
            0 => B0,
            1 => B1,
            2 => B2,
            3 => B3,
            4 => B4,
            5 => B5,
            6 => B6,
            7 => B7,
            8 => B8,
            9 => B9,
            _ => panic!("index must be less than Button::COUNT"),
        }
    }

    /// Returns the [`Button`] at `row`, `col`.
    #[track_caller]
    #[must_use]
    pub const fn from_row_col(row: Row, col: Column) -> Button {
        Button::from_index(row.index() * 5 + col.index())
    }

    /// Returns a random [`Button`].
    ///
    /// See also [`Buttons::choose_n_randomly()`] and [`Buttons::choose_one_randomly()`].
    #[must_use]
    pub fn random() -> Button {
        Buttons::all().choose_one_randomly()
    }

    /// Returns this button's index.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns this button's [`Row`].
    #[must_use]
    pub const fn row(self) -> Row {
        if self.index() < 5 {
            Row::Top
        } else {
            Row::Bottom
        }
    }

    /// Returns this button's [`Column`].
    #[must_use]
    pub const fn col(self) -> Column {
        if self.row().index() == 0 {
            Column::from_index(self.index())
        } else {
            Column::from_index(self.index() - 5)
        }
    }

    /// Return the button clockwise of this button
    /// # Examples
    ///
    /// ```
    /// # use boppo_core::Button;
    /// assert_eq!(Button::B3.next_clockwise(), Button::B4);
    /// assert_eq!(Button::B4.next_clockwise(), Button::B9);
    /// assert_eq!(Button::B5.next_clockwise(), Button::B0);
    /// assert_eq!(Button::B9.next_clockwise(), Button::B8);
    /// ```
    #[must_use]
    pub const fn next_clockwise(self) -> Button {
        use Button::{B0, B1, B2, B3, B4, B5, B6, B7, B8, B9};
        #[allow(
            clippy::missing_panics_doc,
            reason = "These functions will never panic"
        )]
        match self {
            B0 | B1 | B2 | B3 => self.right().unwrap(),
            B4 => B9,
            B5 => B0,
            B6 | B7 | B8 | B9 => self.left().unwrap(),
        }
    }

    /// Return the button counterclockwise of this button
    ///
    /// # Examples
    ///
    /// ```
    /// # use boppo_core::Button;
    /// assert_eq!(Button::B1.next_counterclockwise(), Button::B0);
    /// assert_eq!(Button::B8.next_counterclockwise(), Button::B9);
    /// assert_eq!(Button::B0.next_counterclockwise(), Button::B5);
    /// assert_eq!(Button::B9.next_counterclockwise(), Button::B4);
    /// ```
    #[must_use]
    pub const fn next_counterclockwise(self) -> Button {
        use Button::{B0, B1, B2, B3, B4, B5, B6, B7, B8, B9};
        #[allow(
            clippy::missing_panics_doc,
            reason = "These functions will never panic"
        )]
        match self {
            B0 => B5,
            B1 | B2 | B3 | B4 => self.left().unwrap(),
            B5 | B6 | B7 | B8 => self.right().unwrap(),
            B9 => B4,
        }
    }

    /// The button above this one (if any)
    #[must_use]
    pub const fn above(self) -> Option<Button> {
        if self.index() > 4 {
            Some(Button::from_index(self.index() - 5))
        } else {
            None
        }
    }

    /// The button below this one (if any)
    #[must_use]
    pub const fn below(self) -> Option<Button> {
        if self.index() < 5 {
            Some(Button::from_index(self.index() + 5))
        } else {
            None
        }
    }

    /// The button to the left of this one (if any)
    #[must_use]
    pub const fn left(self) -> Option<Button> {
        if matches!(self, Self::B0 | Self::B5) {
            None
        } else {
            Some(Button::from_index(self.index() - 1))
        }
    }

    /// The button to the right of this one (if any)
    #[must_use]
    pub const fn right(self) -> Option<Button> {
        if matches!(self, Self::B4 | Self::B9) {
            None
        } else {
            Some(Button::from_index(self.index() + 1))
        }
    }

    /// Return the button where this button would be if Boppo is rotated 180 degrees around its center
    #[must_use]
    pub const fn rotate_180(self) -> Button {
        Button::from_index(Button::COUNT - 1 - self.index())
    }

    /// Returns a [`Buttons`] with only this button selected.
    #[must_use]
    pub const fn to_buttons(self) -> Buttons {
        Buttons::from_index(self.index())
    }

    /// Returns a [`Lights`] with only this button's lights selected.
    #[must_use]
    pub const fn to_lights(self) -> Lights {
        Lights::all_from_button(self)
    }

    /// Sets this button's lights to `color`.
    ///
    /// This function sets the lights immediately, if you want to modify many lights' or buttons'
    /// colors at once, consider using [`Framebuffer`][crate::Framebuffer].
    pub fn set_color(self, color: crate::color::RGB) {
        crate::LightsSetter::get().set_color(self.into(), color);
    }

    /// Sets this button's lights to [`color::OFF`][crate::color::OFF]. Shorthand for
    /// `self.set_color(color::OFF)`.
    pub fn set_off(self) {
        self.set_color(crate::color::OFF);
    }

    /// Returns the light at `dir` on this button.
    #[must_use]
    pub const fn light_at(self, dir: LightDir) -> Lights {
        self.to_lights().only(dir)
    }

    /// Returns true if the button is currently pressed.
    ///
    /// See also [`ButtonEvents`][crate::ButtonEvents] which is an alternative way to
    /// receive button events.
    #[must_use]
    pub fn is_pressed(&self) -> bool {
        Buttons::currently_pressed().contains(*self)
    }

    /// Wait for this button to be pressed. If the button is already pressed it returns
    /// immediately.
    pub async fn wait_for_press(&self) {
        self.wait_for(true).await;
    }

    /// Wait for this button to be released. If the button is already released it returns
    /// immediately.
    pub async fn wait_for_release(&self) {
        self.wait_for(false).await;
    }

    async fn wait_for(&self, press: bool) {
        let _ = BUTTON_COUNTS
            .get()
            .unwrap()
            .clone()
            .wait_for(|counts| counts.is_pressed(*self) == press)
            .await;
    }
}

impl<'de> Deserialize<'de> for Button {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let idx = usize::deserialize(deserializer)?;
        if idx > Button::COUNT {
            return Err(D::Error::custom(format!("Invalid button index: {idx}")));
        }
        Ok(Button::from_index(idx))
    }
}

/// A row of [`Button`]s.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Row {
    /// [`Button::B0`] through [`Button::B4`]
    Top = 0,
    /// [`Button::B5`] through [`Button::B9`]
    Bottom = 1,
}

impl Row {
    /// The number of rows of top buttons on Boppo.
    pub const COUNT: usize = 2;

    /// Returns the index of this row, 0 ([`Top`][Row::Top]) or 1 ([`Bottom`][Row::Bottom]).
    #[must_use]
    pub const fn index(&self) -> usize {
        *self as usize
    }

    /// Returns [`Row::Top`] if `idx == 0`, and [`Row::Bottom`] if `idx == 1`.
    ///
    /// # Panics
    ///
    /// This function will panic if `idx` is not `0` or `1`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boppo_core::Row;
    /// let top_row = Row::from_index(0);
    /// assert_eq!(top_row, Row::Top);
    ///
    /// let bottom_row = Row::from_index(1);
    /// assert_eq!(bottom_row, Row::Bottom);
    /// ```
    #[must_use]
    pub const fn from_index(idx: usize) -> Row {
        match idx {
            0 => Row::Top,
            1 => Row::Bottom,
            _ => panic!("Invalid row index"),
        }
    }

    /// Returns the opposite row to this one.
    #[must_use]
    pub const fn opposite(&self) -> Row {
        match self {
            Row::Top => Row::Bottom,
            Row::Bottom => Row::Top,
        }
    }
}

/// A column of [`Buttons`]. When the side buttons face the user,
/// [`Column::C0`] is the leftmost column, and [`Column::C4`] is the rightmost column.
///
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[expect(missing_docs, reason = "Variants documented in enum")]
pub enum Column {
    C0 = 0,
    C1 = 1,
    C2 = 2,
    C3 = 3,
    C4 = 4,
}

impl Column {
    /// The number of columns of top buttons on Boppo.
    pub const COUNT: usize = 5;

    /// Returns the index of this column.
    #[must_use]
    pub const fn index(&self) -> usize {
        *self as usize
    }

    /// Returns the corresponding column for the given `idx`.
    ///
    /// # Panics
    ///
    /// This function will panic if `idx` exceeds `4`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boppo_core::Column;
    /// let col = Column::from_index(2);
    /// assert_eq!(col, Column::C2);
    /// ```
    ///
    /// ```should_panic
    /// # use boppo_core::Column;
    /// // Panic! No fifth column exists!
    /// let col = Column::from_index(5);
    /// ```
    #[must_use]
    pub const fn from_index(idx: usize) -> Column {
        match idx {
            0 => Column::C0,
            1 => Column::C1,
            2 => Column::C2,
            3 => Column::C3,
            4 => Column::C4,
            _ => panic!("Invalid column index"),
        }
    }

    /// Returns an array containing all five [`Column`] variants, ordered left-to-right.
    #[must_use]
    pub const fn all() -> [Column; 5] {
        [Column::C0, Column::C1, Column::C2, Column::C3, Column::C4]
    }
}

#[cfg(test)]
#[path = "./tests/button_test.rs"]
mod test;
