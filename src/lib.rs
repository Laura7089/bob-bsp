//! Board support package for [Bob](https://github.com/yorkaerospace/Bob)
//!
//! The intention with this crate is that all the peripherals available on Bob as a board are
//! available to software, and anything that is not available by default is discarded.
//!
//! Essentially, this is a wrapper around [`rp2040_hal`] and the crates for the sensors available
//! on Bob.
//!
//! # Crate Features
//!
//! - `rev3`: configure for revision 3 of Bob, **incompatible with other revision features**
//! - `rev4`: configure for revision 4 of Bob, **incompatible with other revision features**
//! - `defmt`: enables logging with [`defmt`](https://github.com/knurling-rs/defmt) in both this crate and dependencies
//! - `critical-section`: enables critical-section-based bus sharing for sensors; pair with
//! `critical-section-impl` or provide a different implementation
//! - `critical-section-impl`: enables the
//! [`critical-section`](https://github.com/rust-embedded/critical-section) implementation in
//! [`rp2040-hal`](https://github.com/rp-rs/rp-hal)
//! - `boot2`: link in the second-stage bootloader from
//! [`rp2040-boot2`](https://github.com/rp-rs/rp2040-boot2); requires a correct `*.x` linker
//! script with a `.boot2` section, see the project template (TODO)
//! - `micromath`: enable convenience methods that use the
//! [`micromath`](https://github.com/tarcieri/micromath) crate
#![no_std]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![cfg_attr(not(feature = "boot2"), forbid(unsafe_code))]
#![cfg_attr(feature = "boot2", deny(unsafe_code))]

// TODO: magnetometer
// TODO: accelerometer
// TODO: flash (perhaps an embedded-storage impl?)

// TODO: pass through cortex-m{,-rt} crates?
pub extern crate hp203b;
pub extern crate rp2040_hal;

use fugit::{KilohertzU32, MegahertzU32};
use micromath::F32Ext;
use rp2040_hal as hal;

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

/// Processor clock rate of Bob
pub const CLOCK_RATE: MegahertzU32 = MegahertzU32::MHz(130);

type Pwm2 = hal::pwm::Slice<hal::pwm::Pwm2, <hal::pwm::Pwm2 as hal::pwm::SliceId>::Reset>;

/// Onboard buzzer
pub struct Buzzer {
    pwm: Pwm2,
}

use embedded_hal::pwm::SetDutyCycle;

impl Buzzer {
    /// Create a new [`Buzzer`]
    #[must_use]
    pub fn new(mut pwm: Pwm2, pin: BuzzerPwm) -> Self {
        pwm.channel_b.output_to(pin);
        Self { pwm }
    }

    // TODO
    // pub fn set_volume(&mut self, vol: u8) -> Result<(), E> {
    //     let max = self.pwm.channel_b.get_max_duty_cycle()?;
    //     let desired = (max / 100) * vol as u16;
    //     self.set_duty_cycle(desired)?;
    //     Ok(())
    // }

    // pub fn mute(&mut self) -> Result<(), E> {
    //     self.set_duty_cycle_fully_off()
    // }

    /// Set the frequency for the buzzer
    #[cfg(feature = "micromath")]
    pub fn set_frequency(&mut self, frequency: KilohertzU32) {
        let divider = (CLOCK_RATE.to_kHz() as f32) / (frequency.to_kHz() as f32);

        let div_int = divider.trunc();
        let div_frac = divider.fract() * (2.0.powf(4.0));

        self.pwm.set_div_int(div_int as u8);
        self.pwm.set_div_frac(div_frac as u8);
    }

    /// Set the frequency for the buzzer with raw parts
    ///
    /// The RP2040's PWM slices allow a divider to be set to determine their frequency.
    /// The divider is 8 integer bits and 4 fractional ones, and is divided into the clock rate
    /// of the chip.
    /// On Bob this is [`CLOCK_RATE`].
    pub fn set_frequency_raw(&mut self, div_int: u8, div_frac: u8) {
        self.pwm.set_div_int(div_int);
        self.pwm.set_div_frac(div_frac);
    }

    /// Consume `self` and yield the pin and PWM slice
    pub fn destroy(self) -> (Pwm2, BuzzerPwm) {
        (self.pwm, todo!())
    }
}

/// `i2c0` as exposed on Bob
pub type I2C0 = hal::I2C<hal::pac::I2C0, (I2c0Sda, I2c0Scl)>;

/// [`hp203b::HP203B`] as it appears on Bob
///
/// `I` is generic to allow different types of bus sharing.
pub type Altimeter<I, M> = hp203b::HP203B<I, M, hp203b::csb::CSBHigh>;

macro_rules! get_sensors_impl {
    ($short:ident, $shared:ty, $wrapper:ty) => {
        paste::paste! {
            #[allow(non_camel_case_types)]
            type [<I2C0_ $short __>]<'a> = $shared<'a, I2C0>;

            #[doc = "Initialise all onboard sensors with a`" $shared "`"]
            ///
            /// Returns a tuple of `(altimeter, accelerometer, magnetometer)`.
            /// This is a singleton method - it can only be called once.
            /// For information on sharing and safety, see the `embedded-hal-bus` documentation.
            ///
            /// # Errors
            ///
            /// Forwards errors from [`hp203b::HP203B::new`], TODO and TODO.
            // TODO: make a singleton somehow
            pub fn [<get_sensors_ $short>](
                i2c0: &$wrapper<I2C0>,
                alti_osr: hp203b::OSR,
                alti_channel: hp203b::Channel,
            ) -> Result<
                (Altimeter<[<I2C0_ $short __>], hp203b::mode::Pressure>,),
                <[<I2C0_ $short __>] as embedded_hal::i2c::ErrorType>::Error,
            > {
                let alti = {
                    let new_bus = [<I2C0_ $short __>]::new(i2c0);
                    hp203b::HP203B::new(new_bus, alti_osr, alti_channel)?
                };
                Ok((alti,))
            }
        }
    };
}
use embedded_hal_bus::i2c::{CriticalSectionDevice, RefCellDevice};
get_sensors_impl!(rc, RefCellDevice, core::cell::RefCell);
#[cfg(feature = "critical-section")]
type CSArg<I> = critical_section::Mutex<core::cell::RefCell<I>>;
#[cfg(feature = "critical-section")]
get_sensors_impl!(cs, CriticalSectionDevice, CSArg);

#[cfg(test)]
mod tests {}
