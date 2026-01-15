use std::time::Duration;

use libftd2xx::{BitsPerWord, Ftdi, FtdiCommon, Parity, StopBits, TimeoutError};
use theymx::{Multiverse, UniverseId};

const BAUDRATE: u32 = 250000;
const BITS_8: BitsPerWord = BitsPerWord::Bits8;
const STOP_BITS_2: StopBits = StopBits::Bits2;
const PARITY_NONE: Parity = Parity::No;
const READ_TIMEOUT: Duration = Duration::from_millis(1000);
const WRITE_TIMEOUT: Duration = Duration::from_millis(1000);

pub struct EnttecOpenDmxOutput {
    universe_id: UniverseId,
    ftdi: Ftdi,
}

impl EnttecOpenDmxOutput {
    pub fn new(universe_id: UniverseId, serial_number: &str) -> Result<Self, crate::output::Error> {
        let ftdi = Ftdi::with_serial_number(serial_number)?;

        Ok(Self { universe_id, ftdi })
    }
}

impl super::OutputInstanceImplementation for EnttecOpenDmxOutput {
    fn setup(&mut self) -> Result<(), crate::output::Error> {
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

    fn handle_frame(&mut self, frame: Multiverse) -> Result<(), crate::output::Error> {
        let universe = frame.universe(&self.universe_id).cloned().unwrap_or_default();
        let buffer = universe.values().map(|v| v.as_u8());

        self.ftdi.set_break_on()?;
        self.ftdi.set_break_off()?;

        self.ftdi.write(&[0])?; // We need to add this prefix byte to convert the buffer's 0-index to a 1-index.
        self.ftdi.write_all(&buffer).map_err(|err| match err {
            TimeoutError::FtStatus(ft_status) => crate::output::Error::FtdiError(ft_status),
            TimeoutError::Timeout { .. } => crate::output::Error::Timeout,
        })?;

        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), crate::output::Error> {
        self.ftdi.close()?;

        Ok(())
    }
}
