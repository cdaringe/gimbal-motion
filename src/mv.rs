use libm::roundf;

use crate::motor::MOTOR_MICRO_STEPS_PER_REVOLUTION;

pub struct Move {
    /**
     * Not radians. Sorry bro.
     */
    pub degrees: f32,
    /**
     * Direction. true = forward, false = backward
     */
    pub fwd: bool,
    /**
     * Degrees_per_second
     */
    pub velocity: f32,
}

impl Move {
    pub fn num_steps(&self, drive_teeth: u16, driven_teeth: u16) -> u32 {
        let percent_rev = self.degrees / 360.0;
        let drive_revs_per_driven_rev = driven_teeth as f32 / drive_teeth as f32;
        let drive_steps_f = percent_rev /*  rev */
          * drive_revs_per_driven_rev /* drive_rev / rev */
          * MOTOR_MICRO_STEPS_PER_REVOLUTION /* microsteps / drive_rev */ as f32;
        roundf(drive_steps_f) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_steps() {
        let move_instance = Move {
            degrees: 180.,
            velocity: 100.,
            fwd: true,
        };

        let drive_teeth = 10;
        let driven_teeth = 20;

        let expected_output = /* expected output based on the input values */ 123;
        assert_eq!(
            move_instance.num_steps(drive_teeth, driven_teeth),
            expected_output
        );
    }
}
