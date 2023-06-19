//! Board support package for [Bob](https://github.com/yorkaerospace/Bob) revision 3.
//!
//! The intention with this crate is that all the peripherals available on Bob as a board are
//! available to software, and anything that is not available by default is discarded.
//!
//! Essentially, this is a wrapper around [`rp2040_hal`] and the crates for the sensors available
//! on Bob.
#![no_std]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![cfg_attr(not(feature = "boot2"), forbid(unsafe_code))]
#![cfg_attr(feature = "boot2", deny(unsafe_code))]

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[allow(unsafe_code)]
#[cfg(feature = "boot2")]
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[cfg(all(feature = "rev3", feature = "rev4"))]
compile_error!("Cannot enable both 'rev3' and 'rev4' features simultaneously");

// TODO: magnetometer
// TODO: accelerometer
// TODO: flash (perhaps an embedded-storage impl?)
// TODO: can we do anything with the screw terminal?

// TODO: pass through cortex-m{,-rt} crates?
pub extern crate hp203b;
pub extern crate rp2040_hal;

use rp2040_hal as hal;

hal::bsp_pins! {
    #[allow(missing_docs)]
    Gpio0 { name: gpio0 },
    #[allow(missing_docs)]
    Gpio1 { name: gpio1 },
    #[allow(missing_docs)]
    Gpio2 { name: gpio2 },
    #[allow(missing_docs)]
    Gpio3 { name: gpio3 },
    #[allow(missing_docs)]
    Gpio4 { name: gpio4 },
    #[allow(missing_docs)]
    Gpio26 { name: gpio26 },
    #[allow(missing_docs)]
    Gpio27 { name: gpio27 },
    #[allow(missing_docs)]
    Gpio28 { name: gpio28 },
    #[allow(missing_docs)]
    Gpio29 { name: gpio29 },
    /// Pin connected to the onboard buzzer
    Gpio5 {
        name: buzzer,
        aliases: { FunctionPwm: BuzzerPwm }
    },
    /// Pin used as SDA for [`I2C0`]
    Gpio16 {
        name: i2c0_sda,
        aliases: { FunctionI2C: I2c0Sda }
    },
    /// Pin used as SCL for [`I2C0`]
    Gpio17 {
        name: i2c0_scl,
        aliases: { FunctionI2C: I2c0Scl }
    },
}

type Pwm2 = hal::pwm::Slice<hal::pwm::Pwm2, <hal::pwm::Pwm2 as hal::pwm::SliceId>::Reset>;
type Pwm2B = hal::pwm::Channel<hal::pwm::Pwm2, hal::pwm::FreeRunning, hal::pwm::B>;

/// Onboard buzzer
pub struct Buzzer {
    channel: Pwm2B,
}

use embedded_hal::pwm::SetDutyCycle;

impl Buzzer {
    /// Create a new [`Buzzer`] with a set `frequency`
    pub fn new(mut pwm: Pwm2, pin: BuzzerPwm, frequency: fugit::KilohertzU64) -> Self {
        // TODO: what is the clock rate of Bob?
        pwm.set_div_int(todo!());
        pwm.set_div_frac(todo!());

        let mut channel = pwm.channel_b;
        channel.output_to(pin);
        Self { channel }
    }

    // pub fn set_volume(&mut self, vol: u8) -> Result<(), E> {
    //     let max = self.channel.get_max_duty_cycle()?;
    //     let desired = (max / 100) * vol as u16;
    //     self.set_duty_cycle(desired)?;
    //     Ok(())
    // }

    // pub fn mute(&mut self) -> Result<(), E> {
    //     self.set_duty_cycle_fully_off()
    // }
}

/// `i2c0` as exposed on Bob
pub type I2C0 = hal::I2C<hal::pac::I2C0, (I2c0Sda, I2c0Scl)>;

/// [`hp203b::HP203B`] as it appears on Bob
///
/// `I` is a generic I2C argument to allow different types of bus sharing.
/// See [`sensors_rc`] and TODO.
pub type Altimeter<I, M> = hp203b::HP203B<I, M, hp203b::csb::CSBHigh>;

/// Onboard sensors using [`embedded_hal_bus::i2c::RefCellDevice`]
pub mod sensors_rc {
    use core::cell::RefCell;
    use embedded_hal_bus::i2c::RefCellDevice;

    type I2C0Shared<'a> = RefCellDevice<'a, super::I2C0>;

    /// Initialise all onboard sensors
    ///
    /// This is a singleton method - it can only be called once.
    /// Returns a tuple of `(altimeter, accelerometer, magnetometer)`.
    // TODO: make a singleton somehow
    pub fn get_sensors(
        i2c0: &RefCell<super::I2C0>,
        alti_osr: hp203b::OSR,
        alti_channel: hp203b::Channel,
    ) -> Result<
        (super::Altimeter<I2C0Shared, hp203b::mode::Pressure>,),
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
mod tests {}
