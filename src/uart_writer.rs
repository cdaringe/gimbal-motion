// use {embedded_hal::blocking::serial::Write, esp32_hal::uart::Uart};

use esp_idf_svc::{hal::uart, io::Write, sys::EspError};

pub struct UartWriter<'d> {
    uart: uart::UartTxDriver<'d>,
}

impl<'d> UartWriter<'d> {
    pub fn new(uart: uart::UartTxDriver<'d>) -> Self {
        Self { uart }
    }
}
impl<'d> embedded_hal::blocking::serial::Write<u8> for UartWriter<'d> {
    type Error = EspError;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.uart.write_all(buffer).map_err(|err| err.0)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.uart.wait_done(1000)
    }
}
