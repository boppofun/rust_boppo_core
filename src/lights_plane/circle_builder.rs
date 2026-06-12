use crate::{Framebuffer, Lights, color, lights_plane::LIGHT_LOCATIONS};

/// Builder for drawing a circle of lights on a [`Framebuffer`].
///
/// Created by [`circle`]. Call [`draw`][CircleBuilder::draw] to apply.
pub struct CircleBuilder {
    color: color::RGB,
    center: (f32, f32),
    radius: f32,
    mask: Lights,
    fall_off: Option<f32>,
}

impl CircleBuilder {
    /// Creates a new [`CircleBuilder`] with the given `color`, `center`, and `radius`.
    pub fn new(color: color::RGB, center: (f32, f32), radius: f32) -> Self {
        Self {
            color,
            center,
            radius,
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
    /// Falloff dims lights as they approach the edge of the circle, creating a soft gradient
    /// instead of a hard cutoff. Lights within `distance * radius` of the center are drawn at
    /// full brightness; lights beyond that fade linearly to zero at the circle's edge.
    ///
    /// `distance` is a fraction of the radius in the range `0.0..=1.0`:
    /// - A **smaller** value starts the gradient closer to the center, so more of the circle fades.
    /// - A **larger** value starts the gradient closer to the edge, leaving most of the circle at
    ///   full brightness with only a thin fringe that fades.
    ///
    /// A value of `0.7` is a reasonable starting point.
    #[must_use]
    pub fn relative_fall_off(mut self, distance: f32) -> Self {
        self.fall_off = Some(distance);
        self
    }

    /// Draws the circle onto `fb`. Does not flush `fb`.
    pub fn draw(self, fb: &mut Framebuffer) {
        let dist_within_squared = self.radius * self.radius;
        for idx in self.mask.indices() {
            let (x2, y2) = LIGHT_LOCATIONS[idx];
            let dx = self.center.0 - x2;
            let dy = self.center.1 - y2;
            let dist_squared = dx * dx + dy * dy;
            if dist_squared < dist_within_squared {
                let c = if let Some(fall_off_distance) = self.fall_off {
                    let relative_dist = dist_squared.sqrt() / self.radius;
                    let percent = if relative_dist < fall_off_distance {
                        1.0
                    } else {
                        1. - ((relative_dist - fall_off_distance) / (1.0 - fall_off_distance))
                    };
                    color::dim_to(self.color, percent)
                } else {
                    self.color
                };
                fb.set_color(Lights::from_index(idx), c);
            }
        }
    }
}
