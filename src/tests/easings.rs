use super::*;

const ALL_EASINGS: &[Easing] = &[
    Easing::Linear,
    Easing::SineIn,
    Easing::SineOut,
    Easing::SineInOut,
    Easing::CircularIn,
    Easing::CircularOut,
    Easing::CircularInOut,
    Easing::ExpIn,
    Easing::ExpOut,
    Easing::ExpInOut,
    Easing::ElasticIn,
    Easing::ElasticOut,
    Easing::ElasticInOut,
];

#[test]
fn test_start_end() {
    let mut num_easings = 0;
    for easing in ALL_EASINGS {
        // Add new easings to all_easings above
        println!("Testing easing: {easing:?}");
        #[expect(clippy::match_same_arms, reason = "Easier to keep track of")]
        match easing {
            Easing::Linear => num_easings += 1,
            Easing::SineIn => num_easings += 1,
            Easing::SineOut => num_easings += 1,
            Easing::SineInOut => num_easings += 1,
            Easing::CircularIn => num_easings += 1,
            Easing::CircularOut => num_easings += 1,
            Easing::CircularInOut => num_easings += 1,
            Easing::ExpIn => num_easings += 1,
            Easing::ExpOut => num_easings += 1,
            Easing::ExpInOut => num_easings += 1,
            Easing::ElasticIn => num_easings += 1,
            Easing::ElasticOut => num_easings += 1,
            Easing::ElasticInOut => num_easings += 1,
        }

        // For now all of our easings start and end at 0.0 and 1.0
        #[expect(
            clippy::float_cmp,
            reason = "Easings clamp to 0.0 and 1.0, they must be tested for."
        )]
        {
            assert_eq!(easing.ease(0.0), 0.0);
            assert_eq!(easing.ease(1.0), 1.0);
        }
    }
    assert_eq!(num_easings, ALL_EASINGS.len());
}

fn is_monotonic(easing: Easing) -> bool {
    #[expect(clippy::match_same_arms, reason = "Easier to keep track of")]
    match easing {
        Easing::Linear => true,
        Easing::SineIn => true,
        Easing::SineOut => true,
        Easing::SineInOut => true,
        Easing::CircularIn => true,
        Easing::CircularOut => true,
        Easing::CircularInOut => true,
        Easing::ExpIn => true,
        Easing::ExpOut => true,
        Easing::ExpInOut => true,
        Easing::ElasticIn | Easing::ElasticOut | Easing::ElasticInOut => false,
    }
}

#[test]
fn test_monotonic() {
    let tested_easings = [
        Easing::Linear,
        Easing::SineIn,
        Easing::SineOut,
        Easing::SineInOut,
        Easing::CircularIn,
        Easing::CircularOut,
        Easing::CircularInOut,
        Easing::ExpIn,
        Easing::ExpOut,
        Easing::ExpInOut,
    ];
    let mut num_easings = 0u8;
    for easing in tested_easings {
        // Add new easings to all_easings above
        println!("Testing easing: {easing:?}");
        if is_monotonic(easing) {
            num_easings += 1;
        }

        let mut previous = -1.0;
        for p in 0u8..=100 {
            let p = f32::from(p) / 100.0;
            let next = easing.ease(p);
            assert!(previous < next);
            previous = next;
        }
    }
    assert_eq!(num_easings as usize, tested_easings.len());
}

#[test]
fn test_non_monotonic() {
    let tested_easings = [Easing::ElasticIn, Easing::ElasticOut, Easing::ElasticInOut];
    let mut num_easings = 0;

    for easing in tested_easings {
        println!("Testing easing: {easing:?}");
        if !is_monotonic(easing) {
            num_easings += 1;
        }

        let mut prev = 0.;
        for p in 0u8..=100 {
            let p = f32::from(p) / 100.0;
            let next = easing.ease(p);
            assert!(
                (next - prev).abs() < 0.16,
                "Failed at {p}: abs diff = {}. prev = {prev}, current = {next}",
                (next - prev).abs()
            );
            prev = next;
        }
    }

    assert_eq!(num_easings, tested_easings.len());
}
