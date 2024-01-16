use esp_idf_svc::hal::gpio::IOPin;

use {
    gimbal_motion::gimbal_pins::GimbalBuilder,
};

use {
    esp_idf_svc::{
        hal::{
            delay::FreeRtos,
            gpio::{OutputPin},
            peripherals::Peripherals,
            sys,
        },
        log::EspLogger,
    },
    futures::executor::block_on,
    gimbal_motion::{
        gimbal::{Axis, Gimbal},
        mv::Move,
        server,
        wifi::{connect_wifi, create_wifi},
    },
};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

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
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // let mut z = PinDriver::input(peripherals.pins.gpio9).unwrap();
    // z.set_pull(Pull::Up);
    let tpins = GimbalBuilder::pan_dir(pins.gpio14.downgrade_output().into())
        .pan_step(pins.gpio15.downgrade_output().into())
        .tilt_dir(pins.gpio22.downgrade_output().into())
        .tilt_step(pins.gpio21.downgrade_output().into())
        .pan_endstop(pins.gpio25.downgrade().into())
        .tilt_endstop(pins.gpio26.downgrade().into());

    let mut gimbal = Gimbal::new(tpins, PAN_TEETH, DRIVE_TEETH, TILT_TEETH, DRIVE_TEETH);

    // let gimbal_arc = Arc::new(Mutex::new(gimbal));

    let mut wifi = create_wifi(peripherals.modem)?;
    let ip_info = block_on(connect_wifi(&mut wifi, SSID, PASSWORD))?;
    let _server = server::start(ip_info)?;

    loop {
        gimbal.mv(
            Move {
                degrees: 20.,
                fwd: true,
                velocity: 360. / 20.,
            },
            Axis::Tilt,
        );
        gimbal.mv(
            Move {
                degrees: 20.,
                fwd: true,
                velocity: 360. / 20.,
            },
            Axis::Pan,
        );
        FreeRtos::delay_ms(2000);

        gimbal.mv(
            Move {
                degrees: 20.,
                fwd: false,
                velocity: 360. / 20.,
            },
            Axis::Tilt,
        );
        gimbal.mv(
            Move {
                degrees: 20.,
                fwd: false,
                velocity: 360. / 20.,
            },
            Axis::Pan,
        );

        FreeRtos::delay_ms(1000);
    }
}
