//! Board support package for [Bob](https://github.com/yorkaerospace/Bob) revision 3.
//!
//! The intention with this crate is that all the peripherals available on Bob as a board are
//! available to software, and anything that is not available by default is discarded.
//!
//! Essentially, this is a wrapper around [`rp2040_hal`] and the crates for the sensors available
//! on Bob.
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]

use core::cell::RefCell;
use fugit::RateExtU32;
use rp2040_hal::gpio::{bank0, FunctionI2C, Pin};

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[cfg(feature = "boot2")]
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// TODO: pass through cortex-m{,-rt} crates?

pub extern crate hp203b;
pub extern crate rp2040_hal;

/// `i2c0` as exposed on bob
pub type I2C0 = rp2040_hal::I2C<
    rp2040_hal::pac::I2C0,
    (
        Pin<bank0::Gpio16, FunctionI2C>,
        Pin<bank0::Gpio17, FunctionI2C>,
    ),
>;

type PinDefault<T> = rp2040_hal::gpio::Pin<T, <T as rp2040_hal::gpio::pin::PinId>::Reset>;

#[allow(missing_docs)]
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

/// Singleton struct for peripherals available on Bob
#[allow(missing_docs)]
pub struct Peripherals {
    pub i2c0: RefCell<I2C0>,
    pub gpio: Gpio,
    // TODO: magnetometer
    // TODO: accelerometer
    // TODO: buzzer
    // TODO: flash (perhaps an embedded-storage impl?)
    // TODO: pyro channels
    // TODO: can we do anything with the screw terminal?
}

impl Peripherals {
    // TODO: pass through i2c settings
    /// Take ownership of peripherals
    ///
    /// This can only be called once.
    pub fn take() -> Self {
        let mut perips_base = rp2040_hal::pac::Peripherals::take().unwrap();

        let sio = rp2040_hal::Sio::new(perips_base.SIO);
        let pins = rp2040_hal::gpio::Pins::new(
            perips_base.IO_BANK0,
            perips_base.PADS_BANK0,
            sio.gpio_bank0,
            &mut perips_base.RESETS,
        );

        let i2c0 = RefCell::new(rp2040_hal::I2C::i2c0(
            perips_base.I2C0,
            pins.gpio16.into_mode(),
            pins.gpio17.into_mode(),
            400.kHz(),
            &mut perips_base.RESETS,
            125.MHz(),
        ));

        Self {
            i2c0,
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
        }
    }
}

/// Onboard sensors using [`embedded_hal_bus::i2c::RefCellDevice`]
pub mod sensors_rc {
    use core::cell::RefCell;

    type I2C0Shared<'a> = embedded_hal_bus::i2c::RefCellDevice<'a, super::I2C0>;

    /// The [`hp203b::HP203B`] as it appears on Bob, using `RefCell` bus sharing
    pub type Altimeter<'a, M> = hp203b::HP203B<I2C0Shared<'a>, M, hp203b::csb::CSBHigh>;

    /// Initialise all onboard sensors
    ///
    /// This is a single method - it can only be called once.
    /// Returns a tuple of `(altimeter, accelerometer, magnetometer)`.
    // TODO: make a singleton somehow
    pub fn get_sensors(
        i2c0: &RefCell<super::I2C0>,
        alti_osr: hp203b::OSR,
        alti_channel: hp203b::Channel,
    ) -> Result<
        (Altimeter<hp203b::mode::Pressure>,),
        <I2C0Shared as embedded_hal::i2c::ErrorType>::Error,
    > {
        let alti = {
            let new_bus = I2C0Shared::new(i2c0);
            hp203b::HP203B::new(new_bus, alti_osr, alti_channel)?
        };
        Ok((alti,))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
}
