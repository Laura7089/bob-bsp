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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
}
