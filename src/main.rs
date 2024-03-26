use {
    esp_idf_svc::{
        hal::{
            delay::FreeRtos,
            gpio::{AnyIOPin, IOPin, OutputPin},
            peripherals::Peripherals,
            sys, uart,
        },
        log::EspLogger,
    },
    futures::executor::block_on,
    std::{
        borrow::BorrowMut,
        collections::VecDeque,
        sync::{Arc, Mutex},
    },
};

use gimbal_motion::{cmd::Cmd, gimbal_pins::GimbalBuilder, uart_writer::UartWriter};

use gimbal_motion::{
    gimbal::Gimbal,
    server,
    wifi::{connect_wifi, create_wifi},
};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

/*
 * https://github.com/Rahix/avr-hal/tree/main/examples
 */
const DRIVE_TEETH: u16 = 16;
const TILT_TEETH: u16 = 160;
const PAN_TEETH: u16 = 128;

pub struct TmcRegisters {
    gconf: tmc2209::reg::GCONF,
    vactual: tmc2209::reg::VACTUAL,
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // let motor_conf_coolconf = tmc2209::reg::COOLCONF::default();
    let mut motor_conf_gconf = tmc2209::reg::GCONF::default();
    motor_conf_gconf.set_shaft(true); // spin motor.
    motor_conf_gconf.set_pdn_disable(true);
    let vactual = tmc2209::reg::VACTUAL::default();

    let motor_driver = Arc::new(Mutex::new(
        uart::UartDriver::new(
            peripherals.uart2,
            pins.gpio17,
            pins.gpio16,
            AnyIOPin::none(),
            AnyIOPin::none(),
            &uart::config::Config::new().baudrate(115200.into()),
        )
        .unwrap(),
    ));
    let mut xxx = Arc::clone(&motor_driver);
    let mut bullshit = xxx.lock().unwrap();
    let (mtx, mrx) = bullshit.borrow_mut().split();
    let _ = {
        let mrx_reader = std::thread::spawn(move || {
            let mut tmc_reader = tmc2209::Reader::default();
            let mut buf = [0u8; 256];
            while let Ok(b) = mrx.read(&mut buf, 10000) {
                if let (_, Some(response)) = tmc_reader.read_response(&[b.try_into().unwrap()]) {
                    match response.crc_is_valid() {
                        true => log::info!("Received valid response!"),
                        false => {
                            log::error!("Received invalid response!");
                            continue;
                        }
                    }
                    match response.reg_addr() {
                        Ok(tmc2209::reg::Address::IOIN) => {
                            let reg = response.register::<tmc2209::reg::IOIN>().unwrap();
                            log::info!("{:?}", reg);
                        }
                        Ok(tmc2209::reg::Address::IFCNT) => {
                            let reg = response.register::<tmc2209::reg::IFCNT>().unwrap();
                            log::info!("{:?}", reg);
                        }
                        addr => log::warn!("Unexpected register address: {:?}", addr),
                    }
                }
            }
        });

        let mtx_writer = std::thread::spawn(move || {
            let mut wmtx = UartWriter::new(mtx);
            tmc2209::send_write_request(0, motor_conf_gconf, &mut wmtx).unwrap();
            tmc2209::send_write_request(0, vactual, &mut wmtx).unwrap();
        });

        mrx_reader.join().unwrap();
        mtx_writer.join().unwrap();
    };

    let gimbal_pins = GimbalBuilder::pan_dir(pins.gpio14.downgrade_output().into())
        .pan_step(pins.gpio15.downgrade_output().into())
        .tilt_dir(pins.gpio22.downgrade_output().into())
        .tilt_step(pins.gpio21.downgrade_output().into())
        .pan_endstop(pins.gpio25.downgrade().into())
        .tilt_endstop(pins.gpio26.downgrade().into());

    let cmds_arc: Arc<Mutex<VecDeque<Cmd>>> = Arc::new(Mutex::new(VecDeque::new()));
    let cmds_reader = cmds_arc.clone();

    let gimbal_arc: Arc<Mutex<Gimbal>> = Arc::new(Mutex::new(Gimbal::new(
        gimbal_pins,
        PAN_TEETH,
        DRIVE_TEETH,
        TILT_TEETH,
        DRIVE_TEETH,
        30.,
        30.,
    )));

    let mut wifi = create_wifi(peripherals.modem)?;
    let ip_info = block_on(connect_wifi(&mut wifi, SSID, PASSWORD))?;
    let _server = server::start(ip_info, cmds_arc.clone(), gimbal_arc.clone())?;

    loop {
        let cmd_opt = { cmds_reader.lock().unwrap().borrow_mut().pop_front() };

        if let Some(cmd) = cmd_opt {
            match cmd {
                Cmd::ClearCmdQueue => {
                    let mut cmds = cmds_reader.lock().unwrap();
                    cmds.clear();
                }
                Cmd::ProcessGcode(mv) => {
                    let mut gimbal = gimbal_arc.lock().unwrap();
                    if gimbal.last_error_message.is_none() {
                        match gimbal.process_gcode(mv) {
                            Ok(_) => {}
                            Err(e) => {
                                gimbal.last_error_message = Some(e.to_string());
                                log::error!("failed to process gcode: {e}. restart required");
                            }
                        }
                    }
                }
            }
        }

        FreeRtos::delay_ms(100);
    }
}
