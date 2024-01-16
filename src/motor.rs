pub const MOTOR_MICRO_STEPS_PER_REVOLUTION: u16 = 400 * 8;

// https://www.openimpulse.com/blog/products-page/product-category/42byghm809-stepper-motor-1-68-4-2-kg%E2%8B%85cm/
pub fn steps_per_degree(drive_teeth: u16, driven_teeth: u16) -> f32 {
    let drive_revs_per_driven_rev = f32::from(driven_teeth) / f32::from(drive_teeth);
    let steps_per_rev = f32::from(MOTOR_MICRO_STEPS_PER_REVOLUTION) * drive_revs_per_driven_rev;
    steps_per_rev / 360.0
}
