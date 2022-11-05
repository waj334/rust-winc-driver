use core::any::Any;
pub use atsamd_hal as hal;
use atsamd_hal::clock::GenericClockController;
use atsamd_hal::gpio::{AnyPin, PinId};
use atsamd_hal::sercom::spi::AnySpi;
use atsamd_hal::time::Hertz;
pub use hal::sercom::{
    i2c, spi,
    uart::{self, BaudMode, Oversampling},
    IoSet1, IoSet6, IoSet3,
};
pub use hal::ehal;
pub use hal::pac;
pub use cortex_m::peripheral;
use winc::bsp::{Pin, Spibus};
use crate::board::ehal::digital::v2::OutputPin;

hal::bsp_peripherals!(
    SERCOM3 { I2cSercom }
    SERCOM4 { SpiSercom }
    SERCOM5 { UartSercom }
);

hal::bsp_pins!(
    PA09 {
        name: winc_reset,
        aliases: {
            PushPullOutput: winc_reset,
        }
    }

    PA10 {
        name: winc_cs,
        aliases: {
            PushPullOutput: winc_cs,
        }
    }

    PA11 {
        name: winc_enable,
        aliases: {
            PushPullOutput: winc_enable,
        }
    }

    PA15 {
        name: winc_irq
    }

    PA12 {
        name: spi_sck,
        aliases: {
           AlternateD: Sck
        }
    }

    PA13 {
        name: spi_mosi,
        aliases: {
           AlternateD: Mosi ,
        }
    }

    PA14 {
        name: spi_miso,
        aliases: {
           AlternateD: Miso ,
        }
    }

    PA17 {
        name: status_led
        aliases: {
            PushPullOutput: BuiltinLed,
        }
    }

    PA22 {
        name: i2c_sda,
        aliases: {
            AlternateC: Sda,
        }
    }

    PA23 {
        name: i2c_scl,
        aliases: {
            AlternateC: Scl,
        }
    }

    PB02 {
        name: uart_tx,
        aliases: {
            AlternateD: UartTx
        }
    }

    PB03 {
        name: uart_rx,
        aliases: {
            AlternateD: UartRx
        }
    }
);

pub type I2cPads = i2c::Pads<I2cSercom, IoSet1, Sda, Scl>;
pub type I2c = i2c::I2c<i2c::Config<I2cPads>>;
pub fn i2c_master(
    clocks: &mut GenericClockController,
    baud: impl Into<Hertz>,
    sercom3: I2cSercom,
    mclk: &mut pac::MCLK,
    sda: impl Into<Sda>,
    scl: impl Into<Scl>,
) -> I2c {
    let gclk0 = clocks.gclk0();
    let clock = &clocks.sercom3_core(&gclk0).unwrap();
    let freq = clock.freq();
    let baud = baud.into();
    let pads = i2c::Pads::new(sda.into(), scl.into());
    i2c::Config::new(mclk, sercom3, pads, freq)
        .baud(baud)
        .enable()
}

pub type UartPads = uart::Pads<UartSercom, IoSet6, UartRx, UartTx>;
pub type Uart = uart::Uart<uart::Config<UartPads>, uart::Duplex>;
pub fn uart(
    clocks: &mut GenericClockController,
    baud: impl Into<Hertz>,
    sercom5: UartSercom,
    mclk: &mut pac::MCLK,
    uart_rx: impl Into<UartRx>,
    uart_tx: impl Into<UartTx>,
) -> Uart {
    let gclk0 = clocks.gclk0();
    let clock = &clocks.sercom5_core(&gclk0).unwrap();
    let baud = baud.into();
    let pads = uart::Pads::default().rx(uart_rx.into()).tx(uart_tx.into());
    uart::Config::new(mclk, sercom5, pads, clock.freq())
        .baud(baud, BaudMode::Fractional(Oversampling::Bits16))
        .enable()
}

pub type SpiPads = spi::Pads<SpiSercom, IoSet3, Miso, Mosi, Sck>;
pub type Spi = spi::Spi<spi::Config<SpiPads>, spi::Duplex>;
pub fn spi_master(
    clocks: &mut GenericClockController,
    baud: impl Into<Hertz>,
    sercom4: SpiSercom,

    mclk: &mut pac::MCLK,
    sck: impl Into<Sck>,
    mosi: impl Into<Mosi>,
    miso: impl Into<Miso>,
) -> Spi {
    let gclk0 = clocks.gclk0();
    let clock = clocks.sercom4_core(&gclk0).unwrap();
    let freq = clock.freq();
    let (miso, mosi, sck) = (miso.into(), mosi.into(), sck.into());
    let pads = spi::Pads::default().data_in(miso).data_out(mosi).sclk(sck);
    spi::Config::new(mclk, sercom4, pads, freq)
        .baud(baud)
        .spi_mode(spi::MODE_0)
        .enable()
}

pub struct WINCPin<I: PinId> {
    pub pin: hal::gpio::Pin<I, hal::gpio::PushPullOutput>,
}

impl<I: PinId> WINCPin<I> {
    pub fn new(pin: hal::gpio::Pin<I, hal::gpio::PushPullOutput>) -> WINCPin<I> {
        return WINCPin{pin}
    }
}

impl<I: PinId> Pin for WINCPin<I> {
    fn set_asserted(&mut self, on: bool) {
        if on {
            self.pin.set_high().unwrap();
        } else {
            self.pin.set_low().unwrap();
        }
    }
}


pub struct WINCSpi<SPI: AnySpi, I: PinId>{
    pub spi: SPI,
    pub cs: WINCPin<I>
}

impl<SPI: AnySpi, I: PinId> Spibus<WINCPin<I>> for WINCSpi<SPI, I> {
    fn transfer(&self, input: u8) -> u8 {
        return 0;
    }

    fn cs_pin(&mut self) -> &mut WINCPin<I> {
        return &mut self.cs;
    }
}