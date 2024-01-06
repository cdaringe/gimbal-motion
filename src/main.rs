#![cfg_attr(not(test), no_std)]
#![no_main]
use arduino_hal::hal::usart::BaudrateArduinoExt;
use panic_halt as _;
use turret::{mv::Move, turret as tmod, turret_pins::TurretPins};

/*
 * https://github.com/Rahix/avr-hal/tree/main/examples
 */

const DRIVE_TEETH: u16 = 16;
const TILT_TEETH: u16 = 160;
const PAN_TEETH: u16 = 128;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::Usart::new(
        dp.USART0,
        pins.d0,
        pins.d1.into_output(),
        BaudrateArduinoExt::into_baudrate(57600),
    );

    let tpins = TurretPins {
        pan_dir: pins.d4.into_output().downgrade(),
        pan_step: pins.d6.into_output().downgrade(),
        tilt_dir: pins.d3.into_output().downgrade(),
        tilt_step: pins.d2.into_output().downgrade(),
        led: pins.d13.into_output().downgrade(),
    };

    let mut turret = tmod::Turret::new(tpins, PAN_TEETH, DRIVE_TEETH, TILT_TEETH, DRIVE_TEETH);

    loop {
        turret.pins.led.toggle();
        ufmt::uwriteln!(&mut serial, "out").unwrap();
        turret.mv(
            Move {
                degrees: 25.,
                fwd: true,
                velocity: 360. / 10.,
            },
            tmod::Axis::Tilt,
        );
        ufmt::uwriteln!(&mut serial, "halt-out").unwrap();
        arduino_hal::delay_ms(2000);
        turret.mv(
            Move {
                degrees: 25.,
                fwd: false,
                velocity: 360. / 10.,
            },
            tmod::Axis::Tilt,
        );
        ufmt::uwriteln!(&mut serial, "back").unwrap();
        arduino_hal::delay_ms(2000);
        ufmt::uwriteln!(&mut serial, "halt-back").unwrap();
    }
}
