use crate::{Column, Row};

use super::*;
#[test]
fn test_invert() {
    assert_eq!(Lights::all().invert(), Lights::none());
    let one = Lights::from_index(1);
    let one_inverted = one.invert();
    assert_eq!(one_inverted.len(), 39);
    assert_eq!(one_inverted.as_bitset() & 0x2, 0);
}

#[test]
fn test_indices_reverse() {
    let mut iter = Lights::none().indices();
    assert_eq!(iter.next_back(), None);

    let mut iter = Lights::from_indices([0_usize]).indices();
    assert_eq!(iter.next_back(), Some(0));
    assert_eq!(iter.next_back(), None);

    let mut iter = Lights::all().indices();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next_back(), Some(39));
    assert_eq!(iter.next_back(), Some(38));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next_back(), Some(37));
    assert_eq!(iter.next_back(), Some(36));
    assert_eq!(iter.next(), Some(2));
}

#[test]
fn test_rotate_180() {
    assert_eq!(Lights::from_index(0).rotate_180(), Lights::from_index(39));
    assert_eq!(Lights::from_index(1).rotate_180(), Lights::from_index(38));
    assert_eq!(Lights::from_index(3).rotate_180(), Lights::from_index(36));
    assert_eq!(Lights::from_index(4).rotate_180(), Lights::from_index(35));
    assert_eq!(Lights::from_index(5).rotate_180(), Lights::from_index(34));
    assert_eq!(Lights::from_index(0), Lights::from_index(39).rotate_180());
    assert_eq!(Lights::from_index(1), Lights::from_index(38).rotate_180());
    assert_eq!(Lights::from_index(3), Lights::from_index(36).rotate_180());
    assert_eq!(Lights::from_index(4), Lights::from_index(35).rotate_180());
    assert_eq!(Lights::from_index(5), Lights::from_index(34).rotate_180());
    assert_eq!(
        Lights::from_indices([0, 8]).rotate_180(),
        Lights::from_indices([31, 39])
    );
    assert_eq!(Lights::all().rotate_180(), Lights::all());
    assert_eq!(Lights::none().rotate_180(), Lights::none());
}

#[test]
fn test_from_buttons() {
    let lights: Lights = Buttons::from_index(0).into();
    assert_eq!(lights, Lights::all_from_button(Button::from_index(0)));

    let lights: Lights = Buttons::from_index(9).into();
    assert_eq!(lights, Lights::all_from_button(Button::from_index(9)));

    let lights: Lights = Buttons::column(Column::C0).into();
    assert_eq!(lights, Lights::from_bitset(0b1111_0000_0000_0000_0000_1111));

    let lights: Lights = Buttons::row(Row::Top).into();
    assert_eq!(lights, Lights::from_bitset(0b1111_1111_1111_1111_1111));

    let lights: Lights = Buttons::row(Row::Bottom).into();
    assert_eq!(
        lights,
        Lights::from_bitset(0b1111_1111_1111_1111_1111_0000_0000_0000_0000_0000)
    );

    let lights: Lights = Buttons::all().into();
    assert_eq!(lights, Lights::all());
}
