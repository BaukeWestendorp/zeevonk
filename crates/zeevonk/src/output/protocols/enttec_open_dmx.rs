use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use libftd2xx::{BitsPerWord, Ftdi, FtdiCommon, Parity, StopBits, TimeoutError};
use theymx::{Multiverse, Universe, UniverseId};

use crate::output;

const BAUDRATE: u32 = 250000;
const BITS_8: BitsPerWord = BitsPerWord::Bits8;
const STOP_BITS_2: StopBits = StopBits::Bits2;
const PARITY_NONE: Parity = Parity::No;
const READ_TIMEOUT: Duration = Duration::from_millis(1000);
const WRITE_TIMEOUT: Duration = Duration::from_millis(1000);

const TARGET_OUTPUT_INTERVAL: Duration = Duration::from_millis(25);

pub struct EnttecOpenDmxOutput {
    universe_id: UniverseId,
    serial_number: String,

    most_recent_universe: Arc<RwLock<Option<Universe>>>,

    // On shutdown, we take the handle and join it, leaving `None`.
    worker_handle: Option<JoinHandle<()>>,
}

impl EnttecOpenDmxOutput {
    pub fn new(universe_id: UniverseId, serial_number: String) -> Result<Self, output::Error> {
        Ok(Self {
            universe_id,
            serial_number,
            most_recent_universe: Arc::new(RwLock::new(Some(Universe::new()))),
            worker_handle: None,
        })
    }
}

impl super::OutputInstanceImplementation for EnttecOpenDmxOutput {
    fn setup(&mut self) -> Result<(), output::Error> {
        let ftdi = Ftdi::with_serial_number(&self.serial_number)?;
        let most_recent_universe = Arc::clone(&self.most_recent_universe);
        let worker_handle = thread::spawn(move || worker(ftdi, most_recent_universe));

        self.worker_handle = Some(worker_handle);

        Ok(())
    }

    fn handle_frame(&mut self, frame: Multiverse) -> Result<(), output::Error> {
        let universe = frame.universe(&self.universe_id).cloned().unwrap_or_default();
        *self.most_recent_universe.write().unwrap() = Some(universe);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), output::Error> {
        *self.most_recent_universe.write().unwrap() = None;

        if let Some(handle) = self.worker_handle.take() {
            if let Err(err) = handle.join() {
                log::error!("worker thread join error: {:?}", err);
            }
        }

        Ok(())
    }
}

fn worker(mut ftdi: Ftdi, most_recent_universe: Arc<RwLock<Option<Universe>>>) {
    if let Err(err) = ftdi_init(&mut ftdi) {
        log::error!("FTDI setup failed in worker thread: {:?}", err);
        return;
    }

    loop {
        let buffer = {
            let guard = most_recent_universe.read().unwrap();
            if let Some(universe) = guard.as_ref() {
                universe.values().map(|v| v.as_u8())
            } else {
                if let Err(err) = ftdi_close(&mut ftdi) {
                    log::error!("FTDI close failed in worker thread: {:?}", err);
                }
                return;
            }
        };

        let start_time = std::time::Instant::now();

        if let Err(err) = ftdi_send(&mut ftdi, &buffer) {
            log::error!("error sending buffer to FTDI: {:?}", err);
        }

        let elapsed = start_time.elapsed();
        let target_interval = TARGET_OUTPUT_INTERVAL;
        if elapsed < target_interval {
            thread::sleep(target_interval - elapsed);
        }
    }
}

fn ftdi_init(ftdi: &mut Ftdi) -> Result<(), output::Error> {
    ftdi.reset()?;
    ftdi.set_baud_rate(BAUDRATE)?;
    ftdi.set_data_characteristics(BITS_8, STOP_BITS_2, PARITY_NONE)?;
    ftdi.set_timeouts(READ_TIMEOUT, WRITE_TIMEOUT)?;
    ftdi.set_flow_control_none()?;
    ftdi.clear_rts()?;
    ftdi.purge_rx()?;
    ftdi.purge_tx()?;
    Ok(())
}

fn ftdi_send(ftdi: &mut Ftdi, buffer: &[u8]) -> Result<(), output::Error> {
    ftdi.set_break_on()?;
    ftdi.set_break_off()?;
    ftdi.write(&[0])?; // We need to add this prefix byte to convert the buffer's 0-index to a 1-index.
    ftdi.write_all(buffer).map_err(|err| match err {
        TimeoutError::FtStatus(ft_status) => output::Error::FtdiError(ft_status),
        TimeoutError::Timeout { .. } => output::Error::Timeout,
    })?;
    Ok(())
}

fn ftdi_close(ftdi: &mut Ftdi) -> Result<(), output::Error> {
    ftdi.close()?;
    Ok(())
}
