use super::*;
#[test]
fn test_new_and_accessors() {
    {
        let button = Button::B0;
        let pressed = true;
        let currently_pressed = Buttons::none();
        let evt = ButtonEvent::new(button, pressed, currently_pressed);
        assert_eq!(evt.button(), button);
        assert_eq!(evt.is_pressed(), pressed);
        assert_eq!(evt.all_currently_pressed(), currently_pressed);
    }
    {
        let button = Button::B9;
        let pressed = true;
        let currently_pressed = Buttons::none();
        let evt = ButtonEvent::new(button, pressed, currently_pressed);
        assert_eq!(evt.button(), button);
        assert_eq!(evt.is_pressed(), pressed);
        assert_eq!(evt.all_currently_pressed(), currently_pressed);
    }
    {
        let button = Button::B0;
        let pressed = false;
        let currently_pressed = Buttons::all();
        let evt = ButtonEvent::new(button, pressed, currently_pressed);
        assert_eq!(evt.button(), button);
        assert_eq!(evt.is_pressed(), pressed);
        assert_eq!(evt.all_currently_pressed(), currently_pressed);
    }
}

#[test]
fn button_event_u16_round_trip_preserves_fields() {
    let changed = Button::from_index(3);
    let currently_pressed = Buttons::from_bitset(0b10_0101);
    let event = ButtonEvent::new(changed, true, currently_pressed);
    let round_tripped = ButtonEvent::from_u16(event.as_u16());
    assert_eq!(round_tripped, event);
    assert_eq!(round_tripped.button(), changed);
    assert!(round_tripped.is_pressed());
    assert!(!round_tripped.is_released());
    assert_eq!(
        round_tripped.all_currently_pressed().as_bitset(),
        currently_pressed.as_bitset()
    );
}

#[tokio::test]
async fn test_peek() {
    let (sender, receiver) = tokio::sync::broadcast::channel(10);
    let event = ButtonEvent::new(Button::B0, false, Buttons::none());
    sender.send(event).unwrap();
    let mut button_events = ButtonEvents::from_receiver(receiver);
    assert_eq!(button_events.peek_next().await, event);
    // Should be able to peek the same event multiple times
    assert_eq!(button_events.peek_next().await, event);
    assert_eq!(button_events.peek_next().await, event);
    assert_eq!(button_events.peek_next().await, event);
    assert_eq!(button_events.next().await, event);
    assert_eq!(button_events.try_next(), None);
}
