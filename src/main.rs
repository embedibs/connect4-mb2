//! connect4-mb2
//! ethan dibble <edibble@pdx.edu>

#![no_main]
#![no_std]

use cortex_m_rt::entry;
use critical_section_lock_mut::LockMut;
use embedded_graphics::{
    Drawable,
    image::{Image, ImageRaw},
    pixelcolor::Rgb565,
    prelude::*,
};
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use heapless::Vec;
use microbit::hal::{
    Spim,
    gpio::{self, Floating, Input, Level},
    gpiote::{self, Gpiote},
    pac::{self, interrupt},
    spim::{self, Frequency},
    timer::Timer,
};
use mipidsi::{
    models::GC9A01,
    options::{ColorInversion, Orientation, Rotation},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use oxid8_core::{
    Oxid8,
    display::{BoolVec, DISPLAY_AREA, DISPLAY_WIDTH},
};

mod util;
use util::Button;

const ROM: &[u8] = include_bytes!("../CONNECT4");

#[derive(Default)]
struct Emu(Oxid8);

impl core::ops::Deref for Emu {
    type Target = Oxid8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Emu {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Emu {
    // TODO: make this method better when you have a working example
    // TODO: also try to make the image bigger and more centered
    fn display_as_rgb565(&self) -> Vec<u8, { DISPLAY_AREA * 2 }> {
        self.0
            .display()
            .unpack_as::<BoolVec>()
            .iter()
            .map(|&p| {
                if p {
                    [0b11111111, 0b11111111]
                } else {
                    [0b00000000, 0b00000000]
                }
            })
            .flatten()
            .collect()
    }
}

#[interrupt]
fn GPIOTE() {
    GPIOTE_PERIPHERAL.with_lock(|gpiote| {
        if gpiote.channel0().is_event_triggered() {
            BUTTON_A.with_lock(|btn| btn.handle_event());
        }
        if gpiote.channel1().is_event_triggered() {
            BUTTON_B.with_lock(|btn| btn.handle_event());
        }
        if gpiote.channel2().is_event_triggered() {
            TOUCH.with_lock(|btn| btn.handle_event());
        }
        gpiote.channel0().reset_events();
        gpiote.channel1().reset_events();
        gpiote.channel2().reset_events();
    });
}

static BUTTON_A: LockMut<Button<pac::TIMER1, fn(bool)>> = LockMut::new();
static BUTTON_B: LockMut<Button<pac::TIMER2, fn(bool)>> = LockMut::new();
static TOUCH: LockMut<Button<pac::TIMER3, fn(bool)>> = LockMut::new();

static GPIOTE_PERIPHERAL: LockMut<Gpiote> = LockMut::new();

static EMU: LockMut<Emu> = LockMut::new();

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = microbit::Board::take().unwrap();

    let mut timer0 = Timer::new(board.TIMER0);

    // Setup SPI
    let sck = board.pins.p0_17.into_push_pull_output(Level::Low).degrade();
    let coti = board.pins.p0_13.into_push_pull_output(Level::Low).degrade();

    let dc = board.edge.e08.into_push_pull_output(Level::Low);
    let cs = board.edge.e01.into_push_pull_output(Level::Low);
    let rst = board.edge.e09.into_push_pull_output(Level::High);

    let spi_bus = Spim::new(
        board.SPIM3,
        microbit::hal::spim::Pins {
            sck: Some(sck),
            mosi: Some(coti),
            miso: None,
        },
        Frequency::M32,
        spim::MODE_0,
        0xFF, // ORC overflow character
    );
    let spi = display_interface_spi::SPIInterface::new(
        ExclusiveDevice::new_no_delay(spi_bus, cs).unwrap(),
        dc,
    );

    // Setup GC9A01 display using mipidsi
    let mut display = mipidsi::Builder::new(GC9A01, spi)
        .orientation(Orientation::new().rotate(Rotation::Deg270))
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut timer0)
        .unwrap();

    // Call `embedded_graphics` `clear()` trait method
    <_ as embedded_graphics::draw_target::DrawTarget>::clear(&mut display, Rgb565::BLACK).unwrap();

    ////////////////////////////////////////////////////////////////////////////

    EMU.init(Emu::default());

    EMU.with_lock(|emu| {
        emu.load_font();
        emu.load_rom_bytes(ROM).unwrap();

        let image_data = emu.display_as_rgb565();
        let raw_image = ImageRaw::<Rgb565>::new(image_data.as_slice(), DISPLAY_WIDTH as u32);
        let image = Image::new(&raw_image, Point::zero());

        image.draw(&mut display).unwrap();
    });

    init_buttons(
        board.TIMER1,
        board.TIMER2,
        board.TIMER3,
        board.GPIOTE,
        board.buttons.button_a.degrade(),
        board.buttons.button_b.degrade(),
        board.pins.p1_04.into_floating_input().degrade(),
    );

    init_nvic();

    loop {
        EMU.with_lock(|emu| {
            emu.next_frame().unwrap();

            let image_data = emu.display_as_rgb565();
            let raw_image = ImageRaw::<Rgb565>::new(image_data.as_slice(), DISPLAY_WIDTH as u32);
            let image = Image::new(&raw_image, Point { x: 100, y: 100 });

            image.draw(&mut display).unwrap();
        });

        timer0.delay_ms(200);
    }
}

/// Set up the NVIC to handle interrupts.
fn init_nvic() {
    unsafe { pac::NVIC::unmask(pac::Interrupt::GPIOTE) };
    pac::NVIC::unpend(pac::Interrupt::GPIOTE);
}

/// Set up microbit buttons.
fn init_buttons(
    timer1: pac::TIMER1,
    timer2: pac::TIMER2,
    timer3: pac::TIMER3,
    gpiote: pac::GPIOTE,
    button_a: gpio::Pin<Input<Floating>>,
    button_b: gpio::Pin<Input<Floating>>,
    touch: gpio::Pin<Input<Floating>>,
) {
    let mut timer_debounce_a = Timer::new(timer1);
    let mut timer_debounce_b = Timer::new(timer2);
    let mut timer_debounce_touch = Timer::new(timer3);

    let gpiote = gpiote::Gpiote::new(gpiote);

    // Interrupt any activity on button A
    let _ = gpiote
        .channel0()
        .input_pin(&button_a)
        .toggle()
        .enable_interrupt();

    // Interrupt any activity on button B
    let _ = gpiote
        .channel1()
        .input_pin(&button_b)
        .toggle()
        .enable_interrupt();

    // Interrupt any activity on touch button
    let _ = gpiote
        .channel2()
        .input_pin(&touch)
        .toggle()
        .enable_interrupt();

    GPIOTE_PERIPHERAL.init(gpiote);

    timer_debounce_a.disable_interrupt();
    timer_debounce_a.reset_event();

    BUTTON_A.init(Button::new(button_a, timer_debounce_a, |pressed| {
        EMU.with_lock(|emu| emu.set_key(0x4, pressed));
    }));

    timer_debounce_b.disable_interrupt();
    timer_debounce_b.reset_event();

    BUTTON_B.init(Button::new(button_b, timer_debounce_b, |pressed| {
        EMU.with_lock(|emu| emu.set_key(0x6, pressed));
    }));

    timer_debounce_touch.disable_interrupt();
    timer_debounce_touch.reset_event();

    TOUCH.init(Button::new(touch, timer_debounce_touch, |pressed| {
        EMU.with_lock(|emu| emu.set_key(0x5, pressed));
    }));
}
