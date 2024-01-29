use std::{borrow::BorrowMut, collections::VecDeque, sync::Mutex};

use {esp_idf_svc::hal::gpio::IOPin, std::sync::Arc};

use gimbal_motion::{cmd::Cmd, gimbal_pins::GimbalBuilder};

use {
    esp_idf_svc::{
        hal::{delay::FreeRtos, gpio::OutputPin, peripherals::Peripherals, sys},
        log::EspLogger,
    },
    futures::executor::block_on,
    gimbal_motion::{
        gimbal::{Gimbal},
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

    let cmds_arc: Arc<Mutex<VecDeque<Cmd>>> = Arc::new(Mutex::new(VecDeque::new()));
    let cmds_reader = cmds_arc.clone();

    let mut gimbal = Gimbal::new(
        tpins,
        PAN_TEETH,
        DRIVE_TEETH,
        TILT_TEETH,
        DRIVE_TEETH,
        30.,
        30.,
    );

    // let gimbal_arc = Arc::new(Mutex::new(gimbal));

    let mut wifi = create_wifi(peripherals.modem)?;
    let ip_info = block_on(connect_wifi(&mut wifi, SSID, PASSWORD))?;
    let _server = server::start(ip_info, cmds_arc.clone())?;

    loop {
        let cmd_opt = { cmds_reader.lock().unwrap().borrow_mut().pop_front() };

        if let Some(cmd) = cmd_opt {
            match cmd {
                Cmd::ClearCmdQueue => {
                    let mut cmds = cmds_reader.lock().unwrap();
                    cmds.clear();
                }
                Cmd::ProcessGcode(mv) => gimbal.process_gcode(mv).expect("gcode proc failed"),
            }
        }

        FreeRtos::delay_ms(100);
    }
}
