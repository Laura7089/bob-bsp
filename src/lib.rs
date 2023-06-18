#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use core::cell::RefCell;
use fugit::RateExtU32;
use rp2040_hal::gpio::{bank0, FunctionI2C, Pin};

// TODO: pass through cortex-m{,-rt} crates?

pub extern crate hp203b;
pub extern crate rp2040_hal;

/// i2c0 as exposed on bob
pub type I2C = rp2040_hal::I2C<
    rp2040_hal::pac::I2C0,
    (
        Pin<bank0::Gpio16, FunctionI2C>,
        Pin<bank0::Gpio17, FunctionI2C>,
    ),
>;

type I2CShared<'a> = embedded_hal_bus::i2c::RefCellDevice<'a, I2C>;

pub type Altimeter<'a, M> = hp203b::HP203B<I2CShared<'a>, M, hp203b::csb::CSBHigh>;

type PinDefault<T> = rp2040_hal::gpio::Pin<T, <T as rp2040_hal::gpio::pin::PinId>::Reset>;

/// Exposed breakout pins on bob
pub struct Gpio {
    pub gpio0: PinDefault<bank0::Gpio0>,
    pub gpio1: PinDefault<bank0::Gpio1>,
    pub gpio2: PinDefault<bank0::Gpio2>,
    pub gpio3: PinDefault<bank0::Gpio3>,
    pub gpio4: PinDefault<bank0::Gpio4>,
    pub gpio26: PinDefault<bank0::Gpio26>,
    pub gpio27: PinDefault<bank0::Gpio27>,
    pub gpio28: PinDefault<bank0::Gpio28>,
    pub gpio29: PinDefault<bank0::Gpio29>,
}

// TODO: magnetometer
// TODO: accelerometer
// TODO: buzzer
// TODO: flash (perhaps an embedded-storage impl?)
// TODO: pyro channels
pub struct Peripherals {
    pub i2c: RefCell<I2C>,
    pub gpio: Gpio,
    alti_taken: bool,
}

impl Peripherals {
    // TODO: pass through i2c settings
    pub fn take() -> Self {
        let mut perips_base = rp2040_hal::pac::Peripherals::take().unwrap();

        let sio = rp2040_hal::Sio::new(perips_base.SIO);
        let pins = rp2040_hal::gpio::Pins::new(
            perips_base.IO_BANK0,
            perips_base.PADS_BANK0,
            sio.gpio_bank0,
            &mut perips_base.RESETS,
        );

        let i2c = RefCell::new(rp2040_hal::I2C::i2c0(
            perips_base.I2C0,
            pins.gpio16.into_mode(),
            pins.gpio17.into_mode(),
            400.kHz(),
            &mut perips_base.RESETS,
            125.MHz(),
        ));

        Self {
            i2c,
            gpio: Gpio {
                gpio0: pins.gpio0,
                gpio1: pins.gpio1,
                gpio2: pins.gpio2,
                gpio3: pins.gpio3,
                gpio4: pins.gpio4,
                gpio26: pins.gpio26,
                gpio27: pins.gpio27,
                gpio28: pins.gpio28,
                gpio29: pins.gpio29,
            },
            alti_taken: false,
        }
    }

    pub fn get_altimeter(
        &mut self,
        osr: hp203b::OSR,
        channel: hp203b::Channel,
    ) -> Result<
        Altimeter<'_, hp203b::mode::Pressure>,
        <I2CShared as embedded_hal::i2c::ErrorType>::Error,
    > {
        if self.alti_taken {
            panic!("Tried to take altimeter more than once");
        }
        let new_bus = I2CShared::new(&self.i2c);
        self.alti_taken = true;
        let alti = hp203b::HP203B::new(new_bus, osr, channel)?;
        Ok(alti)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
}
