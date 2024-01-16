use std::{
    num::NonZeroU32,
    sync::{Arc, Mutex},
};

use libm::floorf;

use derive_more::Display;

use log::info;

use esp_idf_svc::hal::{delay::Delay, gpio::Pull, task::notification::Notification};

use crate::{gimbal_pins::GimbalPins, motor::steps_per_degree, mv::Move};

#[derive(Debug, Display)]
pub enum Axis {
    #[display(fmt = "Pan")]
    Pan,
    #[display(fmt = "Tilt")]
    Tilt,
}

pub struct Gimbal {
    pub pins: GimbalPins,
    // (pan, tilt)
    pub pos_steps: (u32, u32),
    pan_teeth: u16,
    tilt_teeth: u16,
    pan_drive_teeth: u16,
    tilt_drive_teeth: u16,
}

impl Gimbal {
    pub fn new(
        pins: GimbalPins,
        pan_teeth: u16,
        pan_drive_teeth: u16,
        tilt_teeth: u16,
        tilt_drive_teeth: u16,
    ) -> Self {
        Self {
            pins,
            pos_steps: (0, 0),
            pan_teeth,
            tilt_teeth,
            pan_drive_teeth,
            tilt_drive_teeth,
        }
    }

    fn steps_per_degree_pan(&self) -> f32 {
        steps_per_degree(self.pan_drive_teeth, self.pan_teeth)
    }

    fn steps_per_degree_tilt(&self) -> f32 {
        steps_per_degree(self.tilt_drive_teeth, self.tilt_teeth)
    }

    pub fn mv(&mut self, mv: Move, axis: Axis) {
        let Move {
            degrees,
            fwd,
            velocity,
        } = mv;
        let sign = if fwd { "+" } else { "-" };

        // calculate how many steps to take
        let steps_per_degree = match &axis {
            Axis::Pan => self.steps_per_degree_pan(),
            Axis::Tilt => self.steps_per_degree_tilt(),
        };

        let num_steps = floorf(degrees * steps_per_degree) as u32;

        let steps_per_second = floorf(
            /* deg / s */ velocity * /* step / deg */ steps_per_degree,
        ) as u32;

        let step_pin = match axis {
            Axis::Pan => &mut self.pins.pan_step,
            Axis::Tilt => &mut self.pins.tilt_step,
        };

        let dir_pin = match axis {
            Axis::Pan => &mut self.pins.pan_dir,
            Axis::Tilt => &mut self.pins.tilt_dir,
        };

        // setup direction
        match (fwd, &axis) {
            (true, Axis::Pan) => dir_pin.high(),
            (true, Axis::Tilt) => dir_pin.high(),
            (false, Axis::Pan) => dir_pin.low(),
            (false, Axis::Tilt) => dir_pin.low(),
        };

        let steps_per_microsecond = (steps_per_second as f32) / (1_000_000.);
        let microseconds_per_step = 1. / steps_per_microsecond;
        let delay_micros = libm::floorf(microseconds_per_step / 2.) as u32;

        let seconds_to_move = (num_steps as f32) / steps_per_second as f32;

        info!("move // axis: {axis}, degrees: {sign}{degrees}, steps: {sign}{num_steps}, delay_micros: {delay_micros}, seconds_to_move: {seconds_to_move:.2}");

        for _i in 0..num_steps {
            step_pin.high();
            Delay::new_default().delay_us(delay_micros);
            step_pin.low();
            Delay::new_default().delay_us(delay_micros);
        }
        self.pos_steps = match &axis {
            Axis::Pan => (self.pos_steps.0 + num_steps, self.pos_steps.1),
            Axis::Tilt => (self.pos_steps.0, self.pos_steps.1 + num_steps),
        };
    }

    pub fn home(&mut self) -> anyhow::Result<()> {
        let max_degrees = 360.;
        let max_pan_steps = (max_degrees * self.steps_per_degree_pan()) as u32;
        self.home_pin(Axis::Pan, max_pan_steps)?;

        let _max_tilt_steps = (max_degrees * self.steps_per_degree_tilt()) as u32;
        Ok(())
    }

    // @todo NOT DONE
    fn home_pin(&mut self, axis: Axis, _max_steps: u32) -> anyhow::Result<()> {
        let endstop_pin = match axis {
            Axis::Pan => &mut self.pins.pan_endstop,
            Axis::Tilt => &mut self.pins.tilt_endstop,
        };
        let _ = endstop_pin.pd.set_pull(Pull::Up);
        let _ = endstop_pin
            .pd
            .set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::LowLevel);
        let notification = Notification::new();
        let notifier = notification.notifier();

        let homed = Arc::new(Mutex::new(false));
        let homed_waiter = homed.clone();
        let homed_notify = homed.clone();

        unsafe {
            endstop_pin.pd.subscribe(move || {
                info!("endstop hit {axis}");
                *homed_notify.lock().unwrap() = true;
                notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
            })?;
        }

        endstop_pin.pd.enable_interrupt()?;

        let mut halted = false;
        while !halted {
            // @todo NOT DONE
            notification.wait(esp_idf_svc::hal::delay::NON_BLOCK);
            Delay::new_default().delay_ms(1);
            {
                halted = *homed_waiter.lock().unwrap();
            }
        }
        Ok(())
    }
}
