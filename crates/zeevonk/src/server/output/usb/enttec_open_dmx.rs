use libftd2xx::{FtStatus, Ftdi, FtdiCommon, StopBits};
use std::time::Duration;
use theymx::Universe;

const BAUDRATE: u32 = 250000;
const BITS_8: libftd2xx::BitsPerWord = libftd2xx::BitsPerWord::Bits8;
const STOP_BITS_2: libftd2xx::StopBits = StopBits::Bits2;
const PARITY_NONE: libftd2xx::Parity = libftd2xx::Parity::No;
const READ_TIMEOUT: Duration = Duration::from_millis(1000);
const WRITE_TIMEOUT: Duration = Duration::from_millis(1000);

pub struct Interface {
    ftdi: Ftdi,
}

impl Interface {
    pub fn new(serial_number: &str) -> Result<Self, FtStatus> {
        let ftdi = Ftdi::with_serial_number(serial_number)?;
        Ok(Self { ftdi })
    }

    pub fn open(&mut self) -> Result<(), FtStatus> {
        self.ftdi.reset()?;
        self.ftdi.set_baud_rate(BAUDRATE)?;
        self.ftdi.set_data_characteristics(BITS_8, STOP_BITS_2, PARITY_NONE)?;
        self.ftdi.set_timeouts(READ_TIMEOUT, WRITE_TIMEOUT)?;
        self.ftdi.set_flow_control_none()?;
        self.ftdi.clear_rts()?;
        self.ftdi.purge_rx()?;
        self.ftdi.purge_tx()?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), FtStatus> {
        self.ftdi.close()?;
        Ok(())
    }

    pub fn write_universe(&mut self, universe: Universe) -> Result<(), FtStatus> {
        self.ftdi.set_break_on()?;
        self.ftdi.set_break_off()?;

        let buffer = universe.values().map(|v| v.as_u8());
        // To make sure we are one indexed.
        self.ftdi.write(&[0]).unwrap();
        self.ftdi.write_all(buffer.as_slice()).unwrap();

        Ok(())
    }
}
