//! Utilities

use embedded_hal::digital::InputPin;
use microbit::hal::{
    gpio::{self, Floating, Input},
    timer::{self, Timer},
};

/// 100ms at 1MHz count rate.
const DEBOUNCE_TIME: u32 = 100 * 1_000_000 / 1000;

/// Debounced button events
pub struct Button<I, F> {
    button: gpio::Pin<Input<Floating>>,
    timer: Timer<I>,
    on_toggle: F,
}

impl<I, F> Button<I, F>
where
    I: timer::Instance,
    F: Fn(bool),
{
    /// # Example
    /// ```ignore
    /// Button::new(button, timer, |pressed| { /* do something */ })
    /// ```
    pub fn new(button: gpio::Pin<Input<Floating>>, timer: Timer<I>, on_toggle: F) -> Self {
        Self {
            button,
            timer,
            on_toggle,
        }
    }

    /// Handle a button event.
    pub fn handle_event(&mut self) {
        if self.timer.read() == 0 {
            (self.on_toggle)(self.button.is_low().unwrap());
            self.timer.start(DEBOUNCE_TIME);
        }
    }
}
