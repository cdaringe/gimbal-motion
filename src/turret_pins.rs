use arduino_hal::{hal::port::Pin, port::mode::Output};

pub struct TurretPins {
    pub pan_dir: Pin<Output>,
    pub pan_step: Pin<Output>,
    pub tilt_dir: Pin<Output>,
    pub tilt_step: Pin<Output>,
    pub led: Pin<Output>,
}
