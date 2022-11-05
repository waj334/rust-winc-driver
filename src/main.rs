#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

mod board;

use winc::driver::WINC;

use core::alloc::Layout;
use alloc_cortex_m::CortexMHeap;
use atsamd_hal::prelude::*;

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::{hprintln};

use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::clock::GenericClockController;

use board::hal;

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

const HEAP_SIZE: usize = 1024; // in bytes

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

    let pins = board::Pins::new(peripherals.PORT);
    let mut status_led = pins.status_led.into_push_pull_output();
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let pa09 =  pins.winc_reset.into_mode::<hal::gpio::PushPullOutput>();
    let pa10 =  pins.winc_cs.into_mode::<hal::gpio::PushPullOutput>();
    let pa11 =  pins.winc_enable.into_mode::<hal::gpio::PushPullOutput>();

    let spi = board::WINCSpi{
        spi: board::spi_master(&mut clocks,
                               24.mhz(),
                               periph_alias!(peripherals.spi_sercom),
                               &mut peripherals.MCLK,
                               pins.spi_sck,
                               pins.spi_mosi,
                               pins.spi_miso),
        cs: board::WINCPin::new(pa10),
    };

    let mut winc_drv = WINC{
        spi,
        enable_pin: board::WINCPin::new(pa11),
        reset_pin: board::WINCPin::new( pa09),
    };

    loop {
        delay.delay_ms(1000u32);
        status_led.set_low().unwrap();
        delay.delay_ms(1000u32);
        hprintln!("blink");
        status_led.set_high().unwrap();
    }
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();

    loop {}
}
