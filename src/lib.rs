#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

// TODO: magnetometer
// TODO: altimeter
// TODO: accelerometer
// TODO: buzzer
// TODO: GPIO breakout
// TODO: i2c (since the contacts are exposed)
// TODO: flash (perhaps an embedded-storage impl?)
// TODO: pass through cortex-m{,-rt} crates?
// TODO: pyro channels

pub use hp203b::{Altitude, Channel as AltiChannel, Pressure, Temperature, HP203B, OSR as AltiOSR};

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
}
