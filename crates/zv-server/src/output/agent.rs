use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use theymx::{Multiverse, UniverseId};
use zv_core::value::AttributeValues;

use crate::output::protocols::OutputInstance;
use crate::project::OutputInstanceDefinition;
use crate::resolver;

pub struct OutputAgent {
    updater: Arc<Updater>,
    update_tx: mpsc::Sender<AttributeValues>,
    update_rx: Mutex<Option<mpsc::Receiver<AttributeValues>>>,
}

impl OutputAgent {
    pub fn new() -> Self {
        let (update_tx, update_rx) = mpsc::channel::<AttributeValues>();
        // We make this a single frame buffer, because we
        // do not need to show late frames. Just the most recent one.
        let (output_tx, output_rx) = crossbeam_channel::bounded::<Multiverse>(1);

        // FIXME: Get from project config.
        let instance_definitions: Vec<OutputInstanceDefinition> =
            vec![OutputInstanceDefinition::EnttecOpenDmx {
                universe_id: UniverseId::new_unchecked(1),
                serial_number: "BG00DNDB".to_string(),
            }];

        for instance_definition in instance_definitions {
            let output_rx = crossbeam_channel::Receiver::clone(&output_rx);

            thread::spawn(move || {
                let maybe_instance = OutputInstance::try_from(instance_definition);
                match maybe_instance {
                    Ok(mut instance) => {
                        if let Err(err) = instance.run(output_rx) {
                            log::error!("output instance errored: {}", err);
                        }
                    }
                    Err(err) => {
                        log::error!("failed to create output instance: {}", err);
                    }
                }
            });
        }

        Self {
            updater: Arc::new(Updater::new(output_tx)),
            update_tx,
            update_rx: Mutex::new(Some(update_rx)),
        }
    }

    // TODO: REMOVE
    pub fn test_send(&self, values: AttributeValues) {
        self.update_tx.send(values).unwrap();
    }

    pub fn start(&self) {
        // Try to take the receiver from the slot. This moves the receiver into
        // the worker thread, avoiding the need to lock the receiver on each tick.
        let rx_opt = { self.update_rx.lock().unwrap().take() };

        let rx = match rx_opt {
            Some(r) => r,
            None => {
                log::warn!("output agent is already running");
                return;
            }
        };

        let updater = Arc::clone(&self.updater);

        thread::spawn(move || {
            let mut multiverse = Multiverse::new();

            let tick_interval = updater.tick_interval;
            let mut deadline = Instant::now() + tick_interval;

            loop {
                let tick_start = Instant::now();

                // Drain all pending updates that arrived since last tick.
                let mut updates = Vec::<AttributeValues>::new();
                loop {
                    match rx.try_recv() {
                        Ok(update) => updates.push(update),
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            log::warn!("update channel disconnected; continuing with local state");
                            break;
                        }
                    }
                }

                // Apply updates to the multiverse.
                if !updates.is_empty() {
                    updater.resolve_updates_into_multiverse(&mut multiverse, &updates);
                }

                updater.transmit_resolved_multiverse(multiverse.clone());

                let tick_end = Instant::now();
                let actual_tick_duration = tick_end.duration_since(tick_start);
                let exec_overrun =
                    actual_tick_duration.checked_sub(tick_interval).unwrap_or(Duration::ZERO);

                // Scheduling.
                let now = Instant::now();
                if now < deadline {
                    spin_sleep::sleep(deadline - now);
                    deadline += tick_interval;

                    log::debug!(
                        "tick ok: dur={:>8.3?} over={:>7.3?}",
                        actual_tick_duration,
                        exec_overrun
                    );
                } else {
                    let deadline_lateness = now.duration_since(deadline);
                    log::warn!(
                        "tick late: dur={:08.3?} over={:08.3?} late={:08.3?}",
                        actual_tick_duration,
                        exec_overrun,
                        deadline_lateness
                    );
                    deadline = now + tick_interval;
                }
            }
        });
    }
}

struct Updater {
    tick_interval: Duration,

    output_tx: crossbeam_channel::Sender<Multiverse>,
}

impl Updater {
    pub fn new(output_tx: crossbeam_channel::Sender<Multiverse>) -> Self {
        Self { tick_interval: Duration::from_secs_f64(1.0 / 44.0), output_tx }
    }

    fn resolve_updates_into_multiverse(
        &self,
        multiverse: &mut Multiverse,
        updates: &[AttributeValues],
    ) {
        for update in updates {
            resolver::resolve(update, multiverse);
        }
    }

    fn transmit_resolved_multiverse(&self, multiverse: Multiverse) {
        let _ = self.output_tx.send(multiverse);
    }
}
