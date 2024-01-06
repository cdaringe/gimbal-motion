use crate::{mv::Move, turret_pins::TurretPins};

// https://www.openimpulse.com/blog/products-page/product-category/42byghm809-stepper-motor-1-68-4-2-kg%E2%8B%85cm/
// @todo is this the correct motor and steps?
pub enum Axis {
    Pan,
    Tilt,
}

pub struct Turret {
    pub pins: TurretPins,
    pub pos: (u16, u16),
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
            pos: (0, 0),
            pan_teeth,
            tilt_teeth,
            pan_drive_teeth,
            tilt_drive_teeth,
        }
    }

    pub fn home() {
        todo!()
    }

    pub fn mv(&mut self, mv: Move, axis: Axis) {
        let (drive_teeth, driven_teeth) = match axis {
            Axis::Pan => (self.pan_drive_teeth, self.pan_teeth),
            Axis::Tilt => (self.tilt_drive_teeth, self.tilt_teeth),
        };
        let drive_steps = mv.num_steps(drive_teeth, driven_teeth);
        match (mv.fwd, axis) {
            (true, Axis::Pan) => self.pins.pan_dir.set_high(),
            (true, Axis::Tilt) => self.pins.tilt_dir.set_high(),
            (false, Axis::Pan) => self.pins.pan_dir.set_low(),
            (false, Axis::Tilt) => self.pins.tilt_dir.set_low(),
        };

        for _ in 0..drive_steps {
            self.pins.tilt_step.set_high();
            self.pins.tilt_step.set_low();
        }

        todo!()
    }
}
