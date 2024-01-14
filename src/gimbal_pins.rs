use esp_idf_svc::hal::gpio::{AnyInputPin, AnyOutputPin, Input, Output, PinDriver};

pub struct GimbalPins {
    pub pan_dir: OutPin,
    pub pan_step: OutPin,
    pub pan_endstop: PinDriver<'static, AnyInputPin, Input>,
    pub tilt_dir: OutPin,
    pub tilt_step: OutPin,
    pub tilt_endstop: PinDriver<'static, AnyInputPin, Input>,
}

pub struct OutPin {
    pub pd: PinDriver<'static, AnyOutputPin, Output>,
}

impl OutPin {
    pub fn high(&mut self) {
        self.pd.set_high().expect("failed to set high");
    }
    pub fn low(&mut self) {
        self.pd.set_high().expect("failed to set high");
    }
}

impl From<AnyOutputPin> for OutPin {
    fn from(value: AnyOutputPin) -> Self {
        OutPin {
            pd: PinDriver::output(value).expect("failed to get pin driver"),
        }
    }
}
