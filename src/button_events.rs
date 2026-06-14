//! Events that occur when a top Button is pressed or released.
//!
//! This module provides the [`ButtonEvents`] and [`ButtonEvent`] structs.
use std::{future::Future, pin::pin};

use std::time::{Duration, Instant};

use log::warn;
use tokio::sync::broadcast::{self, error::TryRecvError};

use crate::{Button, Buttons, internal};

/// A notification that a top button has been pressed or released
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ButtonEvent {
    // From least significant:
    // 4 bits: the button index of the changed button
    // 10 bits: the currently pressed bits (same as Buttons)
    // 1 bit: if the changed button is pressed
    // 1 bit: unused
    data: u16,
}

impl ButtonEvent {
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Button indices are always < u16::MAX"
    )]
    #[doc(hidden)]
    pub fn new(changed: Button, pressed: bool, currently_pressed: Buttons) -> ButtonEvent {
        let data =
            changed.index() as u16 | u16::from(pressed) << 14 | currently_pressed.as_bitset() << 4;
        ButtonEvent { data }
    }

    /// The button whose state changed (i.e. was pressed or released).
    #[must_use]
    pub fn button(&self) -> Button {
        let index = self.data & 0b1111;
        Button::from_index(index as usize)
    }

    /// Is this event a press event?
    #[must_use]
    pub fn is_pressed(&self) -> bool {
        (self.data & (1 << 14)) != 0
    }

    /// Is this event a release event?
    #[must_use]
    pub fn is_released(&self) -> bool {
        !self.is_pressed()
    }

    /// All buttons that were pressed at the time of the event
    #[must_use]
    pub fn all_currently_pressed(&self) -> Buttons {
        Buttons::from_bitset((self.data >> 4) & 0b0011_1111_1111)
    }

    /// Represent this event as an u16 value.
    ///
    /// This is useful for making ButtonEvents cross FFIs.
    #[must_use]
    pub fn as_u16(&self) -> u16 {
        self.data
    }

    /// Make a Button event from a u16 value.
    ///
    /// This is useful for making ButtonEvents cross FFIs.
    pub fn from_u16(data: u16) -> Self {
        Self { data }
    }
}

enum EventTypeFilter {
    Press,
    Release,
    Both,
}

/// A stream of top button press and release events.
///
/// Subscribers should generally poll for events frequently as events are stored
/// in a circular buffer and a subscriber can lose events if too many are
/// generated before being polled.
///
/// See also the following alternative ways to get the pressed/released state of
/// buttons:
/// * [`Buttons::currently_pressed()`]
/// * [`Button::is_pressed()`]
/// * [`Button::wait_for_press()`]
/// * [`Button::wait_for_release()`]
pub struct ButtonEvents {
    raw_events: broadcast::Receiver<ButtonEvent>,
    peeked_event: Option<ButtonEvent>,

    /// Only report events for buttons that are included
    buttons_filter: Buttons,
    /// all events before this are ignored
    ignore_events_until: Option<Instant>,
    /// only report events that match
    event_type_filter: EventTypeFilter,
}

impl ButtonEvents {
    pub(crate) fn from_receiver(
        button_event_receiver: broadcast::Receiver<ButtonEvent>,
    ) -> ButtonEvents {
        ButtonEvents {
            raw_events: button_event_receiver,
            peeked_event: None,
            buttons_filter: Buttons::all(),
            ignore_events_until: None,
            event_type_filter: EventTypeFilter::Both,
        }
    }

    /// Subscribe to all button events that take place after this call to subscribe.
    ///
    /// Events before subscribe will not be delivered.
    ///
    /// # Panics
    ///
    /// This function should ordinarily never panic, but may if the library has not been initialized.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use boppo_core::ButtonEvents;
    /// let mut button_events = ButtonEvents::subscribe();
    /// ```
    pub fn subscribe() -> ButtonEvents {
        ButtonEvents::from_receiver(internal::BUTTON_EVENTS.get().unwrap().subscribe())
    }

    /// Returns a [`Future`] yielding the next [`ButtonEvent`].
    ///
    /// # Panics
    ///
    /// This function should ordinarily never panic, but will if it is called after the internal
    /// [`broadcast::Receiver`] is closed.
    pub async fn next(&mut self) -> ButtonEvent {
        if let Some(peeked_event) = self.peeked_event.take() {
            return peeked_event;
        }
        loop {
            let raw_event = self.raw_events.recv().await;
            match raw_event {
                Ok(raw_event) => {
                    if let Some(event) = self.handle_raw(raw_event) {
                        return event;
                    } // else get the next raw event
                }

                Err(broadcast::error::RecvError::Lagged(num_skipped)) => {
                    warn!("Button event receiver too slow. Dropped {num_skipped} button events.");
                    // Grab the next event. The client will NOT know anything happened.
                    // TODO should we expose a lapped flag or something?
                }
                Err(broadcast::error::RecvError::Closed) => {
                    panic!("Button receiver closed. This is unexpected.")
                }
            }
        }
    }

    /// Waits for either a [`ButtonEvent`] to be received, or for `future` to complete, and returns
    /// whichever is received first.
    ///
    /// # Errors
    ///
    /// This function will return `Err(F::Output)` if `future` completes before a [`ButtonEvent`]
    /// is received.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use boppo_core::ButtonEvents;
    /// let mut button_events = ButtonEvents::subscribe();
    /// let my_future = async {
    ///     sleep_ms(100).await;
    ///     "there have been no button events for 100ms"
    /// };
    ///
    /// match button_events.next_or_output(my_future).await {
    ///     Ok(event) => println!("{event:?}"),
    ///     Err(f_out) => println!("{f_out}"),
    /// }
    /// ```
    pub async fn next_or_output<F>(&mut self, future: F) -> Result<ButtonEvent, F::Output>
    where
        F: Future,
    {
        let future = pin!(future);
        let next_fut = pin!(self.next());
        let either = futures::future::select(future, next_fut).await;
        match either {
            futures::future::Either::Left((output, _future)) => Err(output),
            futures::future::Either::Right((button_event, _next_fut)) => Ok(button_event),
        }
    }

    /// Returns [`Some`] if `self` receives an event before `timeout` elapses. Otherwise,
    /// returns [`None`].
    pub async fn next_or_timeout_after(&mut self, timeout: Duration) -> Option<ButtonEvent> {
        embassy_time::with_timeout(
            embassy_time::Duration::from_micros(timeout.as_micros() as u64),
            self.next(),
        )
        .await
        .ok()
    }

    /// Runs `future` while waiting for a new [`ButtonEvent`], and returns [`Some`] if an event
    /// occurs before `future` completes. Otherwise, this function returns [`None`].
    pub async fn next_or_completed<F>(&mut self, future: F) -> Option<ButtonEvent>
    where
        F: Future,
    {
        self.next_or_output(future).await.ok()
    }

    /// The same as [`ButtonEvents::next_or_output`], but does not consume an event if one is
    /// received.
    ///
    /// # Errors
    ///
    /// This function will return `Err(F::Output)` if `future` completes before a [`ButtonEvent`]
    /// is received.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut button_events = ButtonEvents::subscribe();
    /// let my_future = async {
    ///     sleep_ms(100).await;
    ///     "there have been no button events for 100ms"
    /// };
    ///
    /// match button_events.peek_next_or_output(my_future).await {
    ///     Ok(event) => {
    ///         println!("{event:?}");
    ///         // We know there's an event here; unwrap it.
    ///         let ev = button_events.try_next().unwrap();
    ///         println!("{ev:?}");
    ///         assert_eq!(event, ev);
    ///     },
    ///     Err(f_out) => println!("{f_out}"),
    /// }
    /// ```
    pub async fn peek_next_or_output<F>(&mut self, future: F) -> Result<ButtonEvent, F::Output>
    where
        F: Future,
    {
        let future = pin!(future);
        let next_fut = pin!(self.peek_next());
        let either = futures::future::select(future, next_fut).await;
        match either {
            futures::future::Either::Left((button_event, _future)) => Err(button_event),
            futures::future::Either::Right((output, _next_fut)) => Ok(output),
        }
    }

    /// Like [`next_or_timeout_after`][ButtonEvents::next_or_timeout_after], but peeks the event
    /// instead of consuming it.
    pub async fn peek_next_or_timeout_after(&mut self, timeout: Duration) -> Option<ButtonEvent> {
        embassy_time::with_timeout(
            embassy_time::Duration::from_micros(timeout.as_micros() as u64),
            self.peek_next(),
        )
        .await
        .ok()
    }

    /// Cancels the future if a button event is generated.
    pub async fn peek_next_or_completed<F>(&mut self, future: F) -> Option<ButtonEvent>
    where
        F: Future,
    {
        self.peek_next_or_output(future).await.ok()
    }

    /// Waits for the next [`ButtonEvent`] and returns it without consuming it.
    /// Repeated calls to this function will have the same return value.
    ///
    /// See also [`ButtonEvents::try_peek_next`]
    pub async fn peek_next(&mut self) -> ButtonEvent {
        if let Some(peeked_event) = self.peeked_event {
            return peeked_event;
        }
        let next = self.next().await;
        self.peeked_event = Some(next);
        next
    }

    /// Returns the next [`ButtonEvent`] if one has been received, otherwise returns [`None`].
    ///
    /// # Panics
    ///
    /// This function will panic if the underlying [`broadcast::Receiver`] is closed.
    pub fn try_next(&mut self) -> Option<ButtonEvent> {
        if let Some(peeked_event) = self.peeked_event.take() {
            return Some(peeked_event);
        }
        loop {
            let raw_event = self.raw_events.try_recv();
            match raw_event {
                Ok(raw_event) => {
                    if let Some(event) = self.handle_raw(raw_event) {
                        return Some(event);
                    } // else get the next raw event
                }

                Err(broadcast::error::TryRecvError::Empty) => {
                    return None;
                }

                Err(broadcast::error::TryRecvError::Lagged(num_skipped)) => {
                    warn!("Button event receiver too slow. Dropped {num_skipped} button events.");
                    // Grab the next event. The caller will NOT know anything happened.
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    panic!("Button receiver closed. This is unexpected.")
                }
            }
        }
    }

    /// Returns [`Some`] if an event has been received, without consuming it or waiting.
    /// Otherwise returns [`None`].
    ///
    /// See also [`ButtonEvents::peek_next`]
    pub fn try_peek_next(&mut self) -> Option<ButtonEvent> {
        if let Some(peeked_event) = self.peeked_event {
            return Some(peeked_event);
        }
        let next = self.try_next();
        self.peeked_event = next;
        next
    }

    // return None if the raw event should be filtered
    fn handle_raw(&mut self, event: ButtonEvent) -> Option<ButtonEvent> {
        if let Some(until) = self.ignore_events_until {
            if Instant::now() < until {
                return None;
            }
            self.ignore_events_until = None;
        }
        if !self.buttons_filter.contains(event.button()) {
            return None;
        }

        match self.event_type_filter {
            EventTypeFilter::Both => Some(event),
            EventTypeFilter::Press => Some(event).filter(ButtonEvent::is_pressed),
            EventTypeFilter::Release => Some(event).filter(ButtonEvent::is_released),
        }
    }

    /// Clears all pending button events.
    ///
    /// Any [`ButtonEvent`] returned from a call to [`next`][ButtonEvents::next],
    /// [`peek_next`][ButtonEvents::peek_next], etc via `self` are guaranteed to
    /// have been received *after* this function returns.
    pub fn clear_pending(&mut self) {
        self.peeked_event.take();
        while let Ok(_) | Err(TryRecvError::Lagged(_)) = self.raw_events.try_recv() {}
    }

    /// Only press events will be returned from next.
    ///
    /// Released events will still be reflected in the state of `currently_pressed`.
    pub fn only_report_press_events(&mut self) {
        if let Some(event) = self.peeked_event
            && event.is_released()
        {
            self.peeked_event.take();
        }

        self.event_type_filter = EventTypeFilter::Press;
    }

    /// Report both press and release events.
    pub fn report_press_and_release_events(&mut self) {
        self.event_type_filter = EventTypeFilter::Both;
    }

    /// Only release events will be returned from next.
    pub fn only_report_release_events(&mut self) {
        if let Some(event) = self.peeked_event
            && event.is_pressed()
        {
            self.peeked_event.take();
        }

        self.event_type_filter = EventTypeFilter::Release;
    }

    /// If one or more press events (not release events) follows a press event within `duration`, do not report them.
    pub fn ignore_presses_within(&mut self, duration: Duration) {
        self.peeked_event.take();
        self.ignore_events_until = Some(Instant::now() + duration);
    }

    /// Reset to the default behavior of not ignoring press events within a duration specified to `ignore_presses_within`.
    pub fn remove_press_within_filter(&mut self) {
        self.ignore_events_until = None;
    }

    /// Filter out events that don't come from a button in `buttons`.
    pub fn filter_by_buttons(&mut self, buttons: Buttons) {
        if let Some(event) = self.peeked_event
            && !buttons.contains(event.button())
        {
            self.peeked_event.take();
        }

        self.buttons_filter = buttons;
    }

    /// Stop filtering by button. This is the same as `filter_by_buttons(Buttons::all())`.
    pub fn remove_button_filter(&mut self) {
        self.filter_by_buttons(Buttons::all());
    }
}

#[cfg(test)]
#[path = "./tests/button_events_test.rs"]
mod test;
