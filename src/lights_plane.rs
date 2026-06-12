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

use crate::Framebuffer;
use crate::{Button, LightDir, Lights, color};

mod circle_builder;
mod coordinates;

pub use circle_builder::CircleBuilder;
pub use {coordinates::BUTTON_LOCATIONS, coordinates::LIGHT_LOCATIONS};

/// Returns a [`CircleBuilder`] for drawing a circle at `(x, y)` with the given `color` and `radius`.
///
/// Coordinates are defined in the module documentation.
///
/// By default all lights within the radius are drawn at full brightness. Call
/// [`fall_off`][CircleBuilder::fall_off] to enable edge dimming.
///
/// Call [`draw`][CircleBuilder::draw] on the builder to apply it to a [`Framebuffer`].
///
/// # Examples
///
/// ```no_run
/// # use boppo_core::{Framebuffer, Lights, color};
/// # use boppo_core::lights_plane::circle;
/// # let mut fb = Framebuffer::default();
/// // Solid circle, all lights:
/// circle(color::RED, (0.0, 0.0), 3.0).draw(&mut fb);
///
/// // Soft-edged circle — inner 70% full brightness, outer 30% fades:
/// circle(color::RED, (0.0, 0.0), 3.0).fall_off(0.7).draw(&mut fb);
///
/// // Solid circle masked to top row:
/// circle(color::BLUE, (0.0, 1.0), 2.0).mask(Lights::top_row()).draw(&mut fb);
/// ```
#[must_use]
pub fn circle(color: color::RGB, center: (f32, f32), radius: f32) -> CircleBuilder {
    CircleBuilder::new(color, center, radius)
}

/// Draws a rectangle of a certain color and dimensions on button lights.
///
/// This function assumes x and y are the *bottom left corner* of the rectangle, and width and height are positive.
///
/// Coordinates are defined in the module documentation.
///
/// Frame buffer needs to be flushed for lights to be updated.
pub fn draw_light_rectangle_with_mask(
    color: color::RGB,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fb: &mut Framebuffer,
    mask: Lights,
) {
    for idx in mask.indices() {
        const FALL_OFF_DISTANCE: f32 = 0.2;
        let (l_x, l_y) = coordinates_for_light_index(idx);
        let distance_to_nearest_border = distance_within_rectangle(l_x, l_y, x, y, width, height);
        let alpha = (distance_to_nearest_border / FALL_OFF_DISTANCE).clamp(0., 1.);
        fb.set_color(Lights::from_index(idx), color::dim_to(color, alpha));
    }
}

/// Draws a rectangle of a certain color and dimensions on frambuffer `fb`.
///
/// This function assumes x and y are the *bottom left corner* of the rectangle, and width and height are positive.
///
/// Coordinates are defined in the module documentation.
///
/// Frame buffer needs to be flushed for lights to be updated.
pub fn draw_light_rectangle(
    color: color::RGB,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fb: &mut Framebuffer,
) {
    draw_light_rectangle_with_mask(color, x, y, width, height, fb, Lights::all());
}

/// Returns the x location of `col`.
#[must_use]
pub fn x_val_for_col(col: crate::Column) -> f32 {
    match col {
        crate::Column::C0 => -4.0,
        crate::Column::C1 => -2.0,
        crate::Column::C2 => 0.0,
        crate::Column::C3 => 2.0,
        crate::Column::C4 => 4.0,
    }
}

/// Returns the x location of `row`.
#[must_use]
pub fn y_val_for_row(row: crate::Row) -> f32 {
    match row {
        crate::Row::Top => 1.0,
        crate::Row::Bottom => -1.0,
    }
}

/// Returns the `x, y` coordinates of `button`.
///
/// See also [`BUTTON_LOCATIONS`].
#[must_use]
pub const fn coordinates_for_button(button: Button) -> (f32, f32) {
    BUTTON_LOCATIONS[button.index()]
}

/// Returns the `x, y` coordinates of `button`'s `dir` light.
///
/// See also [`LIGHT_LOCATIONS`].
#[must_use]
pub fn coordinates_for_light(button: Button, dir: LightDir) -> (f32, f32) {
    LIGHT_LOCATIONS[button.index() * 4 + dir as usize]
}

/// Returns the `x, y` coordinates of the light at `index`.
///
/// See also [`LIGHT_LOCATIONS`].
#[must_use]
pub fn coordinates_for_light_index(index: usize) -> (f32, f32) {
    LIGHT_LOCATIONS[index]
}

/// Returns the inner distance between a point the nearest border of a given rectangle.
/// Returns 0 if the point is outside the rectangle.
fn distance_within_rectangle(
    p_x: f32,
    p_y: f32,
    rect_x: f32,
    rect_y: f32,
    rect_width: f32,
    rect_height: f32,
) -> f32 {
    // If point within rectangle
    if p_x > rect_x && p_x < rect_x + rect_width && p_y > rect_y && p_y < rect_y + rect_height {
        f32::min(
            f32::min((p_y - rect_y - rect_height).abs(), (p_y - rect_y).abs()),
            f32::min((p_x - rect_x - rect_width).abs(), (p_x - rect_x).abs()),
        )
    } else {
        // We don't care for points outside the rectangle, so distance is considered 0
        0.0
    }
}
