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
use crate::{Button, LightDir, Lights, color::Color};

mod coordinates;

pub use {coordinates::BUTTON_LOCATIONS, coordinates::LIGHT_LOCATIONS};

/// Draws a circle of a certain color and radius on button lights.
///
/// Coordinates are defined in the module documentation.
///
/// Frame buffer needs to be flushed for lights to be updated.
pub fn draw_light_circle_with_mask<C: Color + Copy>(
    c: C,
    x: f32,
    y: f32,
    radius: f32,
    fb: &mut Framebuffer<C>,
    mask: Lights,
) {
    const FALL_OFF_DISTANCE: f32 = 0.7;

    let dist_within_squared = radius * radius;
    for idx in mask.indices() {
        let (x2, y2) = LIGHT_LOCATIONS[idx];
        let dx = x - x2;
        let dy = y - y2;
        let dist_squared = dx * dx + dy * dy;
        let within = dist_squared < dist_within_squared;
        if within {
            let dist = dist_squared.sqrt();
            let relative_dist = dist / radius;
            let percent_colored = if relative_dist < FALL_OFF_DISTANCE {
                1.0
            } else {
                1. - ((relative_dist - FALL_OFF_DISTANCE) / (1.0 - FALL_OFF_DISTANCE))
            };
            let this_color = c.dim_to(percent_colored);
            fb.set_color(Lights::from_index(idx), this_color);
        }
    }
}

// TODO: make a builder pattern for drawing circles that takes in all of the options
// TODO: Remove _light_ from these names
// TODO: make the fallover color and distance controllable and also allow the falloff color
// to be the existing color of the framebuffer
/// Draws a `c`-colored, radius `radius`, circle at `x, y` onto `fb`, ignoring any lights not
/// selected by `mask`.
/// Does not flush `fb`.
pub fn draw_light_circle_no_fall_off<C: Color + Copy>(
    c: C,
    x: f32,
    y: f32,
    radius: f32,
    fb: &mut Framebuffer<C>,
    mask: Lights,
) {
    let dist_within_squared = radius * radius;
    for idx in mask.indices() {
        let (x2, y2) = LIGHT_LOCATIONS[idx];
        let dx = x - x2;
        let dy = y - y2;
        let dist_squared = dx * dx + dy * dy;
        let within = dist_squared < dist_within_squared;
        if within {
            fb.set_color(Lights::from_index(idx), c);
        }
    }
}

/// Draws a rectangle of a certain color and dimensions on button lights.
///
/// This function assumes x and y are the *bottom left corner* of the rectangle, and width and height are positive.
///
/// Coordinates are defined in the module documentation.
///
/// Frame buffer needs to be flushed for lights to be updated.
pub fn draw_light_rectangle_with_mask<C: Color + Copy>(
    color: C,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fb: &mut Framebuffer<C>,
    mask: Lights,
) {
    for idx in mask.indices() {
        const FALL_OFF_DISTANCE: f32 = 0.2;
        let (l_x, l_y) = coordinates_for_light_index(idx);
        let distance_to_nearest_border = distance_within_rectangle(l_x, l_y, x, y, width, height);
        let alpha = (distance_to_nearest_border / FALL_OFF_DISTANCE).clamp(0., 1.);
        fb.set_color(Lights::from_index(idx), color.dim_to(alpha));
    }
}

/// Draws a circle of a certain color and radius on button lights.
///
/// Coordinates are defined in the module documentation.
///
/// Frame buffer needs to be flushed for lights to be updated.
pub fn draw_light_circle<C: Color + Copy>(
    c: C,
    x: f32,
    y: f32,
    radius: f32,
    fb: &mut Framebuffer<C>,
) {
    draw_light_circle_with_mask(c, x, y, radius, fb, Lights::all());
}

/// Draws a rectangle of a certain color and dimensions on frambuffer `fb`.
///
/// This function assumes x and y are the *bottom left corner* of the rectangle, and width and height are positive.
///
/// Coordinates are defined in the module documentation.
///
/// Frame buffer needs to be flushed for lights to be updated.
pub fn draw_light_rectangle<C: Color + Copy>(
    color: C,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fb: &mut Framebuffer<C>,
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
