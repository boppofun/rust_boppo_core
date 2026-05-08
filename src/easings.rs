use std::f32::consts::PI;

/// An [`Easing`] function affects how a value transitions (from 0.0 to 1.0)
///
/// `In` describes an easing that speeds up as it gets closer to 1.0.
/// `Out` describes an easing that slows down as it gets closer to 1.0.
/// `InOut` describes an easing that starts slow, speeds up in the middle, and ends slow.
///
/// See easings.net for graphical examples.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[expect(
    missing_docs,
    reason = "In/Out/InOut described in enum documentation. Not feasible to use text to describe variants."
)]
pub enum Easing {
    Linear,
    SineIn,
    SineOut,
    SineInOut,
    CircularIn,
    CircularOut,
    CircularInOut,
    ExpIn,
    ExpOut,
    ExpInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
}

impl Easing {
    /// Determine what percent `value` is between `start` and `end`
    #[must_use]
    pub fn percent(value: f32, start: f32, end: f32) -> f32 {
        (value / (end - start)).clamp(0., 1.)
    }

    /// Given an input between 0.0 and 1.0, return an output (normally) between 0.0 and 1.0.
    ///
    /// How the input transition changes the output transition is based on the Easing variant.
    #[must_use]
    pub fn ease(&self, percent: f32) -> f32 {
        const C4: f32 = std::f32::consts::FRAC_PI_3 * 2.;
        const C5: f32 = std::f32::consts::TAU / 4.5;

        debug_assert!(
            (0.0..=1.0).contains(&percent),
            "percent must be between 0.0 and 1.0"
        );

        match self {
            Easing::Linear => percent,
            Easing::SineIn => 1. - (PI * percent * 0.5).cos(),
            Easing::SineOut => (PI * percent * 0.5).sin(),
            Easing::SineInOut => -((PI * percent).cos() - 1.) * 0.5,
            Easing::CircularIn => 1. - (1. - percent.powi(2)).sqrt(),
            Easing::CircularOut => (1. - (percent - 1.).powi(2)).sqrt(),
            Easing::CircularInOut => {
                if percent < 0.5 {
                    (1. - (1. - (2. * percent).powi(2)).sqrt()) * 0.5
                } else {
                    ((1. - (-2. * percent + 2.).powi(2)).sqrt() + 1.) * 0.5
                }
            }
            Easing::ExpIn => {
                if percent == 0. {
                    0.
                } else {
                    2f32.powf(10. * percent - 10.)
                }
            }
            Easing::ExpOut =>
            {
                #[expect(
                    clippy::float_cmp,
                    reason = "`Easing::percent` `clamp`s `percent` to 0..=1, so 1.0 will be reached exactly eventually"
                )]
                if percent == 1. {
                    1.
                } else {
                    1.0 - 2f32.powf(-10. * percent)
                }
            }
            Easing::ExpInOut => match percent {
                0. | 1. => percent,
                ..=0.5 => 2f32.powf(20. * percent - 10.) * 0.5,
                _ => (2. - 2f32.powf(-20. * percent + 10.)) * 0.5,
            },
            Easing::ElasticIn => match percent {
                0. | 1. => percent,
                _ => -2f32.powf(10. * percent - 10.) * ((percent * 10. - 10.75) * C4).sin(),
            },
            Easing::ElasticOut => match percent {
                0. | 1. => percent,
                _ => 2f32.powf(-10. * percent) * ((percent * 10. - 0.75) * C4).sin() + 1.,
            },
            Easing::ElasticInOut => match percent {
                0. | 1. => percent,
                ..0.5 => {
                    -(2f32.powf(20. * percent - 10.) * ((20. * percent - 11.125) * C5).sin()) * 0.5
                }
                _ => {
                    (2f32.powf(-20. * percent + 10.) * ((20. * percent - 11.125) * C5).sin()) * 0.5
                        + 1.
                }
            },
        }
    }
}

#[cfg(test)]
#[path = "./tests/easings.rs"]
mod test;
