use crate::{Framebuffer, Lights, color, lights_plane::LIGHT_LOCATIONS};

/// Builder for drawing a rectangle of lights on a [`Framebuffer`].
///
/// Created by [`rectangle`][super::rectangle]. Call [`draw`][RectangleBuilder::draw] to apply.
pub struct RectangleBuilder {
    color: color::RGB,
    bottom_left: (f32, f32),
    width: f32,
    height: f32,
    mask: Lights,
    fall_off: Option<f32>,
}

impl RectangleBuilder {
    /// Creates a new [`RectangleBuilder`] with the given `color`, `bottom_left` corner, `width`, and `height`.
    pub fn new(color: color::RGB, bottom_left: (f32, f32), width: f32, height: f32) -> Self {
        Self {
            color,
            bottom_left,
            width,
            height,
            mask: Lights::all(),
            fall_off: None,
        }
    }

    /// Restricts which lights are eligible to be drawn. Defaults to [`Lights::all`].
    #[must_use]
    pub fn mask(mut self, mask: Lights) -> Self {
        self.mask = mask;
        self
    }

    /// Enables edge falloff with a configurable `distance`.
    ///
    /// Falloff dims lights near the edges of the rectangle, creating a soft gradient instead of a
    /// hard cutoff. Lights farther than `distance` from any edge are drawn at full brightness;
    /// lights within `distance` of an edge fade linearly to zero at the border.
    ///
    /// `distance` is in the same coordinate units as the rectangle dimensions. A value of `0.2`
    /// is a reasonable starting point; larger values widen the fade zone.
    #[must_use]
    pub fn fall_off(mut self, distance: f32) -> Self {
        self.fall_off = Some(distance);
        self
    }

    /// Draws the rectangle onto `fb`. Does not flush `fb`.
    pub fn draw(self, fb: &mut Framebuffer) {
        for idx in self.mask.indices() {
            let light_pos = LIGHT_LOCATIONS[idx];
            let alpha = if let Some(fall_off_distance) = self.fall_off {
                let inner_dist =
                    distance_within_rectangle(light_pos, self.bottom_left, self.width, self.height);
                (inner_dist / fall_off_distance).clamp(0., 1.)
            } else if point_within_rectangle(light_pos, self.bottom_left, self.width, self.height) {
                1.0
            } else {
                0.0
            };
            if alpha > 0.0 {
                fb.set_color(Lights::from_index(idx), color::dim_to(self.color, alpha));
            }
        }
    }
}

/// Returns whether a point is strictly inside a rectangle.
fn point_within_rectangle(p: (f32, f32), bottom_left: (f32, f32), width: f32, height: f32) -> bool {
    let (p_x, p_y) = p;
    let (rect_x, rect_y) = bottom_left;
    p_x > rect_x && p_x < rect_x + width && p_y > rect_y && p_y < rect_y + height
}

/// Returns the inner distance from a point to the nearest border of a rectangle.
/// Returns 0 if the point is outside or on the border.
fn distance_within_rectangle(
    p: (f32, f32),
    bottom_left: (f32, f32),
    width: f32,
    height: f32,
) -> f32 {
    if !point_within_rectangle(p, bottom_left, width, height) {
        return 0.0;
    }
    let (p_x, p_y) = p;
    let (rect_x, rect_y) = bottom_left;
    f32::min(
        f32::min((p_y - rect_y - height).abs(), (p_y - rect_y).abs()),
        f32::min((p_x - rect_x - width).abs(), (p_x - rect_x).abs()),
    )
}
