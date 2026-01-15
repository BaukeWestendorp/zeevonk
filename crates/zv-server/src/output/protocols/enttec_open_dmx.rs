use std::time::Duration;

use libftd2xx::{BitsPerWord, Ftdi, FtdiCommon, Parity, StopBits};
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
    pub fn new(universe_id: UniverseId, serial_number: &str) -> Self {
        let ftdi = Ftdi::with_serial_number(serial_number).unwrap();

        Self { universe_id, ftdi }
    }
}

impl super::OutputInstanceImplementation for EnttecOpenDmxOutput {
    fn setup(&mut self) {
        self.ftdi.reset().unwrap();
        self.ftdi.set_baud_rate(BAUDRATE).unwrap();
        self.ftdi.set_data_characteristics(BITS_8, STOP_BITS_2, PARITY_NONE).unwrap();
        self.ftdi.set_timeouts(READ_TIMEOUT, WRITE_TIMEOUT).unwrap();
        self.ftdi.set_flow_control_none().unwrap();
        self.ftdi.clear_rts().unwrap();
        self.ftdi.purge_rx().unwrap();
        self.ftdi.purge_tx().unwrap();
    }

    fn handle_frame(&mut self, frame: Multiverse) {
        let universe = frame.universe(&self.universe_id).cloned().unwrap_or_default();
        let buffer = universe.values().map(|v| v.as_u8());

        self.ftdi.set_break_on().unwrap();
        self.ftdi.set_break_off().unwrap();

        self.ftdi.write(&[0]).unwrap(); // We need this to conver the buffer's 0-index to a 1-index.
        self.ftdi.write_all(&buffer).unwrap();
    }

    fn shutdown(&mut self) {
        self.ftdi.close().unwrap();
    }
}
