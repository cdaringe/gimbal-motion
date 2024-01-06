#![cfg_attr(not(test), no_std)]
#![no_main]

/*
 * https://github.com/Rahix/avr-hal/tree/main/examples
 */

const DRIVE_TEETH: u16 = 16;
const TILT_TEETH: u16 = 160;
const PAN_TEETH: u16 = 128;

use panic_halt as _;

use turret::{turret as tmod, turret_pins::TurretPins};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let tpins = TurretPins {
        pan_dir: pins.d1.into_output().downgrade(),
        pan_step: pins.d2.into_output().downgrade(),
        tilt_dir: pins.d3.into_output().downgrade(),
        tilt_step: pins.d4.into_output().downgrade(),
        led: pins.d13.into_output().downgrade(),
    };

    let mut turret = tmod::Turret::new(tpins, PAN_TEETH, DRIVE_TEETH, TILT_TEETH, DRIVE_TEETH);

    loop {
        turret.pins.led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
