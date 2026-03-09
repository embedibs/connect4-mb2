//! connect4-mb2
//! ethan dibble <edibble@pdx.edu>

#![no_main]
#![no_std]

use cortex_m_rt::entry;
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
    gpio::Level,
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

    let mut emu = Emu::default();

    emu.load_font();
    emu.load_rom_bytes(ROM).unwrap();

    let image_data = emu.display_as_rgb565();
    let raw_image = ImageRaw::<Rgb565>::new(image_data.as_slice(), DISPLAY_WIDTH as u32);
    let image = Image::new(&raw_image, Point::zero());

    image.draw(&mut display).unwrap();

    loop {
        emu.next_frame().unwrap();

        let image_data = emu.display_as_rgb565();
        let raw_image = ImageRaw::<Rgb565>::new(image_data.as_slice(), DISPLAY_WIDTH as u32);
        let image = Image::new(&raw_image, Point { x: 100, y: 100 });

        image.draw(&mut display).unwrap();

        timer0.delay_ms(500);
        rprintln!("loop");
    }
}
