use libm::floorf;

use crate::{motor::steps_per_degree, mv::Move, turret_pins::TurretPins};

// https://www.openimpulse.com/blog/products-page/product-category/42byghm809-stepper-motor-1-68-4-2-kg%E2%8B%85cm/
// @todo is this the correct motor and steps?
pub enum Axis {
    Pan,
    Tilt,
}

pub struct Turret {
    pub pins: TurretPins,
    // (pan, tilt)
    pub pos_steps: (u32, u32),
    pan_teeth: u16,
    tilt_teeth: u16,
    pan_drive_teeth: u16,
    tilt_drive_teeth: u16,
}

impl Turret {
    pub fn new(
        pins: TurretPins,
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

    pub fn mv(&mut self, mv: Move, axis: Axis) -> u32 {
        // setup direction
        match (mv.fwd, &axis) {
            (true, Axis::Pan) => self.pins.pan_dir.set_high(),
            (true, Axis::Tilt) => self.pins.tilt_dir.set_high(),
            (false, Axis::Pan) => self.pins.pan_dir.set_low(),
            (false, Axis::Tilt) => self.pins.tilt_dir.set_low(),
        };
        // calculate how many steps to take
        let steps_per_degree = match &axis {
            Axis::Pan => self.steps_per_degree_pan(),
            Axis::Tilt => self.steps_per_degree_tilt(),
        };
        let num_steps = floorf(mv.degrees * steps_per_degree) as u32;
        // move those steps
        let steps_per_second = floorf(
            /* deg / s */ mv.velocity * /* step / deg */ steps_per_degree,
        ) as u32;
        let delay_micros = self.mv_steps(num_steps, steps_per_second, &axis);
        self.pos_steps = match &axis {
            Axis::Pan => (self.pos_steps.0 + num_steps, self.pos_steps.1),
            Axis::Tilt => (self.pos_steps.0, self.pos_steps.1 + num_steps),
        };
        delay_micros
    }

    fn mv_steps(&mut self, steps: u32, steps_per_second: u32, axis: &Axis) -> u32 {
        let pin = match axis {
            Axis::Pan => &mut self.pins.pan_step,
            Axis::Tilt => &mut self.pins.tilt_step,
        };
        let steps_per_microsecond = (steps_per_second as f32) / (1_000_000.);
        let microseconds_per_step = 1. / steps_per_microsecond;
        let delay_micros = libm::floorf(microseconds_per_step / 2.) as u32;
        for _ in 0..steps {
            pin.set_high();
            arduino_hal::delay_us(delay_micros);
            pin.set_low();
            arduino_hal::delay_us(delay_micros);
        }
        delay_micros
    }
}
