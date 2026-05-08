use super::*;
#[test]
fn test_above() {
    assert_eq!(Button::B0.above(), None);
    assert_eq!(Button::B1.above(), None);
    assert_eq!(Button::B2.above(), None);
    assert_eq!(Button::B3.above(), None);
    assert_eq!(Button::B4.above(), None);
    assert_eq!(Button::B5.above(), Some(Button::B0));
    assert_eq!(Button::B6.above(), Some(Button::B1));
    assert_eq!(Button::B7.above(), Some(Button::B2));
    assert_eq!(Button::B8.above(), Some(Button::B3));
    assert_eq!(Button::B9.above(), Some(Button::B4));
}

#[test]
fn test_below() {
    assert_eq!(Button::B0.below(), Some(Button::B5));
    assert_eq!(Button::B1.below(), Some(Button::B6));
    assert_eq!(Button::B2.below(), Some(Button::B7));
    assert_eq!(Button::B3.below(), Some(Button::B8));
    assert_eq!(Button::B4.below(), Some(Button::B9));
    assert_eq!(Button::B5.below(), None);
    assert_eq!(Button::B6.below(), None);
    assert_eq!(Button::B7.below(), None);
    assert_eq!(Button::B8.below(), None);
    assert_eq!(Button::B9.below(), None);
}

#[test]
fn test_left() {
    assert_eq!(Button::B0.left(), None);
    assert_eq!(Button::B1.left(), Some(Button::B0));
    assert_eq!(Button::B2.left(), Some(Button::B1));
    assert_eq!(Button::B3.left(), Some(Button::B2));
    assert_eq!(Button::B4.left(), Some(Button::B3));
    assert_eq!(Button::B5.left(), None);
    assert_eq!(Button::B6.left(), Some(Button::B5));
    assert_eq!(Button::B7.left(), Some(Button::B6));
    assert_eq!(Button::B8.left(), Some(Button::B7));
    assert_eq!(Button::B9.left(), Some(Button::B8));
}

#[test]
fn test_right() {
    assert_eq!(Button::B0.right(), Some(Button::B1));
    assert_eq!(Button::B1.right(), Some(Button::B2));
    assert_eq!(Button::B2.right(), Some(Button::B3));
    assert_eq!(Button::B3.right(), Some(Button::B4));
    assert_eq!(Button::B4.right(), None);
    assert_eq!(Button::B5.right(), Some(Button::B6));
    assert_eq!(Button::B6.right(), Some(Button::B7));
    assert_eq!(Button::B7.right(), Some(Button::B8));
    assert_eq!(Button::B8.right(), Some(Button::B9));
    assert_eq!(Button::B9.right(), None);
}
