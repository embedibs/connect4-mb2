//! Utilities

use microbit::hal::timer::{self, Timer};

/// 100ms at 1MHz count rate.
const DEBOUNCE_TIME: u32 = 100 * 1_000_000 / 1000;

/// Debounced button events
pub struct Button<I, F> {
    timer: Timer<I>,
    on_press: F,
}

impl<I, F> Button<I, F>
where
    I: timer::Instance,
    F: Fn(),
{
    pub fn new(timer: Timer<I>, on_press: F) -> Self {
        Self { timer, on_press }
    }

    pub fn handle_event(&mut self) {
        if self.timer.read() == 0 {
            (self.on_press)();
            self.timer.start(DEBOUNCE_TIME);
        }
    }
}
