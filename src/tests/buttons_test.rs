use super::*;
#[test]
fn test_invert() {
    assert_eq!(Buttons::row(Row::Top).invert(), Buttons::row(Row::Bottom));
    assert_eq!(Buttons::all().invert(), Buttons::none());
}

#[test]
fn test_indices_rows() {
    assert_eq!(
        Buttons::row(Row::Top).indices().collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4]
    );
    assert_eq!(
        Buttons::row(Row::Bottom).indices().collect::<Vec<_>>(),
        vec![5, 6, 7, 8, 9]
    );
}

#[test]
fn test_indices_column() {
    assert_eq!(
        Buttons::column(Column::C0).indices().collect::<Vec<_>>(),
        vec![0, 5]
    );
    assert_eq!(
        Buttons::column(Column::C2).indices().collect::<Vec<_>>(),
        vec![2, 7]
    );
    assert_eq!(
        Buttons::column(Column::C4).indices().collect::<Vec<_>>(),
        vec![4, 9]
    );
}

#[test]
fn test_indices_reverse() {
    let mut iter = Buttons::none().indices();
    assert_eq!(iter.next_back(), None);

    let mut iter = Buttons::from_indices([0_usize]).indices();
    assert_eq!(iter.next_back(), Some(0));
    assert_eq!(iter.next_back(), None);

    let mut iter = Buttons::all().indices();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next_back(), Some(9));
    assert_eq!(iter.next_back(), Some(8));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next_back(), Some(7));
    assert_eq!(iter.next_back(), Some(6));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next_back(), Some(5));
    assert_eq!(iter.next_back(), Some(4));
    assert_eq!(iter.next_back(), Some(3));
    assert_eq!(iter.next_back(), None);
}

#[test]
fn test_rotate_180() {
    assert_eq!(Buttons::from_index(0).rotate_180(), Buttons::from_index(9));
    assert_eq!(Buttons::from_index(1).rotate_180(), Buttons::from_index(8));
    assert_eq!(Buttons::from_index(3).rotate_180(), Buttons::from_index(6));
    assert_eq!(Buttons::from_index(4).rotate_180(), Buttons::from_index(5));
    assert_eq!(Buttons::from_index(5).rotate_180(), Buttons::from_index(4));
    assert_eq!(
        Buttons::from_indices([0, 8]).rotate_180(),
        Buttons::from_indices([1, 9])
    );
    assert_eq!(Buttons::all().rotate_180(), Buttons::all());
    assert_eq!(Buttons::none().rotate_180(), Buttons::none());
}
