use esp_idf_svc::hal::{
    delay::FreeRtos,
    gpio::{InputPin, OutputPin, PinDriver},
    peripherals::Peripherals,
    sys,
};
use gimbal_test::{
    gimbal::{Axis, Gimbal},
    gimbal_pins::GimbalPins,
    mv::Move,
};

/*
 * https://github.com/Rahix/avr-hal/tree/main/examples
 */
const DRIVE_TEETH: u16 = 16;
const TILT_TEETH: u16 = 160;
const PAN_TEETH: u16 = 128;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    sys::link_patches();
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // let mut serial = arduino_hal::Usart::new(
    //     dp.USART0,
    //     pins.d0,
    //     pins.d1.into_output(),
    //     BaudrateArduinoExt::into_baudrate(57600),
    // );

    let tpins = GimbalPins {
        pan_dir: PinDriver::output(pins.gpio14.downgrade_output()).expect("obtained pin"),
        pan_step: PinDriver::output(pins.gpio15.downgrade_output()).expect("obtained pin"),
        pan_endstop: PinDriver::input(pins.gpio25.downgrade_input()).expect("obtained pin"),
        tilt_dir: PinDriver::output(pins.gpio22.downgrade_output()).expect("obtained pin"),
        tilt_step: PinDriver::output(pins.gpio21.downgrade_output()).expect("obtained pin"),
        tilt_endstop: PinDriver::input(pins.gpio26.downgrade_input()).expect("obtained pin"),
        // led: pins.d13.into_output().downgrade(),
    };

    let mut gimbal = Gimbal::new(tpins, PAN_TEETH, DRIVE_TEETH, TILT_TEETH, DRIVE_TEETH);

    loop {
        // gimbal.pins.led.toggle();
        // ufmt::uwriteln!(&mut serial, "out").unwrap();
        gimbal.mv(
            Move {
                degrees: 25.,
                fwd: true,
                velocity: 360. / 10.,
            },
            Axis::Tilt,
        );
        gimbal.mv(
            Move {
                degrees: 25.,
                fwd: true,
                velocity: 360. / 10.,
            },
            Axis::Pan,
        );
        // ufmt::uwriteln!(&mut serial, "halt-out").unwrap();
        FreeRtos::delay_ms(2000);
        gimbal.mv(
            Move {
                degrees: 25.,
                fwd: false,
                velocity: 360. / 10.,
            },
            Axis::Tilt,
        );
        gimbal.mv(
            Move {
                degrees: 25.,
                fwd: false,
                velocity: 360. / 10.,
            },
            Axis::Pan,
        );
        // ufmt::uwriteln!(&mut serial, "back").unwrap();
        // arduino_hal::delay_ms(2000);
        FreeRtos::delay_ms(2000);
        // ufmt::uwriteln!(&mut serial, "halt-back").unwrap();
    }
}
