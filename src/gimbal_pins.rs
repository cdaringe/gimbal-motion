use esp_idf_svc::hal::gpio::{AnyInputPin, AnyOutputPin, Input, Output, PinDriver};

pub struct GimbalPins {
    pub pan_dir: PinDriver<'static, AnyOutputPin, Output>,
    pub pan_step: PinDriver<'static, AnyOutputPin, Output>,
    pub pan_endstop: PinDriver<'static, AnyInputPin, Input>,
    pub tilt_dir: PinDriver<'static, AnyOutputPin, Output>,
    pub tilt_step: PinDriver<'static, AnyOutputPin, Output>,
    pub tilt_endstop: PinDriver<'static, AnyInputPin, Input>,
}
