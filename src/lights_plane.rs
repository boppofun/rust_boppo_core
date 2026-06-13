//! Utilities based on laying out the button lights on a cartesian plane.
//!
//! Coordiantes are defined as follows:
//!
//! * (0, 0) is the center of the tablet
//! * The center of the Buttons on [`Row::Top`][crate::Row::Top] are at y
//!   coordinate 1 and on [`Row::Bottom`][crate::Row::Bottom] are at y coordinate -1
//! * Button center to button center (both vertically and horizontally) is 2.0
//! * A button is about 0.75 units in diameter
//!
//! This means, for example, the center of the following buttons are located as follow:
//!
//! * [`Button::B2`] is above the origin at (0,1)
//! * [`Button::B7`] is below the origin at coordinates (0,-1)
//! * [`Button::B0`] (top left) is at coordinates (-4,1)
//! * [`Button::B9`] (bottom right) is at coordinates (4,-1).

use crate::{Button, LightDir, color};

mod circle_builder;
mod coordinates;
mod rectangle_builder;

pub use circle_builder::CircleBuilder;
pub use rectangle_builder::RectangleBuilder;
pub use {coordinates::BUTTON_LOCATIONS, coordinates::LIGHT_LOCATIONS};

/// Returns a [`CircleBuilder`] for drawing a circle at `(x, y)` with the given `color` and `radius`.
///
/// Coordinates are defined in the module documentation.
///
/// By default all lights within the radius are drawn at full brightness. Call
/// [`relative_fall_off`][CircleBuilder::relative_fall_off] to enable edge dimming.
///
/// Call [`draw`][CircleBuilder::draw] on the builder to apply it to a
/// [`Framebuffer`][crate::Framebuffer].
///
/// # Examples
///
/// ```no_run
/// # use boppo_core::{Buttons, Framebuffer, Lights, Row, color};
/// # use boppo_core::lights_plane::circle;
/// # let mut fb = Framebuffer::default();
/// // Solid circle, all lights:
/// circle(color::RED, (0.0, 0.0), 3.0).draw(&mut fb);
///
/// // Soft-edged circle — inner 70% full brightness, outer 30% fades:
/// circle(color::RED, (0.0, 0.0), 3.0).relative_fall_off(0.7).draw(&mut fb);
///
/// // Solid circle masked to top row:
/// circle(color::BLUE, (0.0, 1.0), 2.0).mask(Buttons::row(Row::Top).into()).draw(&mut fb);
/// ```
#[must_use]
pub fn circle(color: color::RGB, center: (f32, f32), radius: f32) -> CircleBuilder {
    CircleBuilder::new(color, center, radius)
}

/// Returns a [`RectangleBuilder`] for drawing a rectangle at `bottom_left` with the given `color`,
/// `width`, and `height`.
///
/// `bottom_left` is the bottom-left corner of the rectangle. Width and height must be positive.
///
/// Coordinates are defined in the module documentation.
///
/// By default all lights within the rectangle are drawn at full brightness. Call
/// [`fall_off`][RectangleBuilder::fall_off] to enable edge dimming.
///
/// Call [`draw`][RectangleBuilder::draw] on the builder to apply it to a
/// [`Framebuffer`][crate::Framebuffer].
///
/// # Examples
///
/// ```no_run
/// # use boppo_core::{Buttons, Framebuffer, Lights, Row, color};
/// # use boppo_core::lights_plane::rectangle;
/// # let mut fb = Framebuffer::default();
/// // Solid rectangle, all lights:
/// rectangle(color::RED, (-2.0, -1.0), 4.0, 2.0).draw(&mut fb);
///
/// // Soft-edged rectangle — 0.2 unit fade zone at each edge:
/// rectangle(color::RED, (-2.0, -1.0), 4.0, 2.0).fall_off(0.2).draw(&mut fb);
///
/// // Solid rectangle masked to bottom row:
/// rectangle(color::BLUE, (-2.0, -1.0), 4.0, 2.0).mask(Buttons::row(Row::Bottom).into()).draw(&mut fb);
/// ```
#[must_use]
pub fn rectangle(
    color: color::RGB,
    bottom_left: (f32, f32),
    width: f32,
    height: f32,
) -> RectangleBuilder {
    RectangleBuilder::new(color, bottom_left, width, height)
}

/// Returns the x location of `col`.
#[must_use]
pub fn column_location(col: crate::Column) -> f32 {
    match col {
        crate::Column::C0 => -4.0,
        crate::Column::C1 => -2.0,
        crate::Column::C2 => 0.0,
        crate::Column::C3 => 2.0,
        crate::Column::C4 => 4.0,
    }
}

/// Returns the y location of `row`.
#[must_use]
pub fn row_location(row: crate::Row) -> f32 {
    match row {
        crate::Row::Top => 1.0,
        crate::Row::Bottom => -1.0,
    }
}

/// Returns the `x, y` coordinates of `button`.
///
/// See also [`BUTTON_LOCATIONS`].
#[must_use]
pub const fn button_location(button: Button) -> (f32, f32) {
    BUTTON_LOCATIONS[button.index()]
}

/// Returns the `x, y` coordinates of `button`'s `dir` light.
///
/// See also [`LIGHT_LOCATIONS`].
#[must_use]
pub fn light_location(button: Button, dir: LightDir) -> (f32, f32) {
    LIGHT_LOCATIONS[button.index() * 4 + dir as usize]
}
