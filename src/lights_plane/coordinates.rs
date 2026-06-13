use crate::{Button, Lights};

const UNIT_IN_MM: f32 = 39. / 2.;
// button center to center of LED: 9 mm
// button center to edge of button: 14.0 mm
//
// Its not clear where to consider the LED since it diffuses light. Since it
// diffuses light all the way to the button edge but also to the center of the
// button. Right now we choose a point closer to the edge of the button but we
// might want to re-evaluate
const LIGHT_TO_BUTTON_CENTER_MM: f32 = 13.0;
pub const LIGHT_TO_BUTTON_CENTER: f32 = LIGHT_TO_BUTTON_CENTER_MM / UNIT_IN_MM;

/// X,Y locations for each [`Button`][`crate::Button`], by their index.
pub const BUTTON_LOCATIONS: [(f32, f32); Button::COUNT] = [
    (-4.0, 1.0),
    (-2.0, 1.0),
    (0.0, 1.0),
    (2.0, 1.0),
    (4.0, 1.0),
    (-4.0, -1.0),
    (-2.0, -1.0),
    (0.0, -1.0),
    (2.0, -1.0),
    (4.0, -1.0),
];

macro_rules! light_locations {
    ($($i : expr),+) => {
        [
            $(
            (
                BUTTON_LOCATIONS[$i].0,
                BUTTON_LOCATIONS[$i].1 + LIGHT_TO_BUTTON_CENTER,
            ), // TOP
            (
                BUTTON_LOCATIONS[$i].0 - LIGHT_TO_BUTTON_CENTER,
                BUTTON_LOCATIONS[$i].1,
            ), // LEFT
            (
                BUTTON_LOCATIONS[$i].0 + LIGHT_TO_BUTTON_CENTER,
                BUTTON_LOCATIONS[$i].1,
            ), // RIGHT
            (
                BUTTON_LOCATIONS[$i].0,
                BUTTON_LOCATIONS[$i].1 - LIGHT_TO_BUTTON_CENTER,
            ), // BOTTOM
            )+
        ]
    };
}

/// X,Y locations for each [`Light`][`crate::Lights`], by their index.
pub const LIGHT_LOCATIONS: [(f32, f32); Lights::COUNT] =
    light_locations![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

#[cfg(test)]
mod tests {
    use crate::{Buttons, LightDir, Lights};

    use super::*;

    /// Verify button locations according to a centric origin with 1 unit being 1 half
    /// of the distance between buttons
    #[test]
    fn verify_button_locations() {
        for (idx, _) in Buttons::all().into_iter().enumerate() {
            match idx {
                0..5 => {
                    assert_eq!(BUTTON_LOCATIONS[idx].0, (idx as i32 * 2 - 4) as f32);
                    assert_eq!(BUTTON_LOCATIONS[idx].1, 1.0);
                }
                5..10 => {
                    assert_eq!(BUTTON_LOCATIONS[idx].0, ((idx as i32 - 5) * 2 - 4) as f32);
                    assert_eq!(BUTTON_LOCATIONS[idx].1, -1.0);
                }
                _ => {
                    unreachable!("Button indexes should not be over 10");
                }
            }
        }
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "Testing known values")]
    fn verify_relative_light_locations() {
        for (idx, _) in Lights::all().into_iter().enumerate() {
            let button_idx = idx / 4;
            let light_dir = LightDir::from_index(idx % 4).expect("Light dir is somehow wrong, this is likely due to an error introduced in LightDir::from_index");
            match light_dir {
                LightDir::Top => {
                    assert_eq!(LIGHT_LOCATIONS[idx].0, BUTTON_LOCATIONS[button_idx].0);
                    assert_eq!(
                        LIGHT_LOCATIONS[idx].1,
                        BUTTON_LOCATIONS[button_idx].1 + LIGHT_TO_BUTTON_CENTER
                    );
                }
                LightDir::Left => {
                    assert_eq!(
                        LIGHT_LOCATIONS[idx].0,
                        BUTTON_LOCATIONS[button_idx].0 - LIGHT_TO_BUTTON_CENTER
                    );
                    assert_eq!(LIGHT_LOCATIONS[idx].1, BUTTON_LOCATIONS[button_idx].1);
                }
                LightDir::Right => {
                    assert_eq!(
                        LIGHT_LOCATIONS[idx].0,
                        BUTTON_LOCATIONS[button_idx].0 + LIGHT_TO_BUTTON_CENTER
                    );
                    assert_eq!(LIGHT_LOCATIONS[idx].1, BUTTON_LOCATIONS[button_idx].1);
                }
                LightDir::Bottom => {
                    assert_eq!(LIGHT_LOCATIONS[idx].0, BUTTON_LOCATIONS[button_idx].0);
                    assert_eq!(
                        LIGHT_LOCATIONS[idx].1,
                        BUTTON_LOCATIONS[button_idx].1 - LIGHT_TO_BUTTON_CENTER
                    );
                }
            }
        }
    }
}
