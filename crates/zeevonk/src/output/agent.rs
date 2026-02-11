use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use theymx::{Address, Multiverse};
use thread_priority::ThreadBuilderExt;

use crate::output::protocols::OutputInstance;
use crate::project::Project;
use crate::resolver;
use crate::value::AttributeValues;

pub struct OutputAgent {
    updater: Arc<Updater>,
    update_tx: mpsc::Sender<OutputAgentUpdate>,
    update_rx: Mutex<Option<mpsc::Receiver<OutputAgentUpdate>>>,
}

impl OutputAgent {
    pub fn new(project: Arc<Project>) -> Self {
        let (update_tx, update_rx) = mpsc::channel::<OutputAgentUpdate>();
        // We make this a single frame buffer, because we
        // do not need to show late frames. Just the most recent one.
        let (output_tx, output_rx) = crossbeam_channel::bounded::<Multiverse>(1);

        let instance_definitions = project.file().dmx_output.instances.clone();

        for (ix, instance_definition) in instance_definitions.into_iter().enumerate() {
            let output_rx = crossbeam_channel::Receiver::clone(&output_rx);

            thread::Builder::new()
                .name(format!("instance_{}", ix))
                .spawn_with_priority(thread_priority::ThreadPriority::Max, move |prio_result| {
                    assert!(prio_result.is_ok());

                    match OutputInstance::try_from(instance_definition) {
                        Ok(mut instance) => {
                            if let Err(err) = instance.run(output_rx) {
                                log::error!("output instance errored: {}", err);
                            }
                        }
                        Err(err) => {
                            log::error!("failed to create output instance: {}", err);
                        }
                    }
                })
                .expect("should spawn output instance thread");
        }

        Self {
            updater: Arc::new(Updater::new(output_tx, project)),
            update_tx,
            update_rx: Mutex::new(Some(update_rx)),
        }
    }

    pub fn set_attribute_values(&self, values: AttributeValues) {
        self.update_tx.send(OutputAgentUpdate::SetAttributeValues(values)).unwrap();
    }

    pub fn set_highlighted_values(
        &self,
        highlighted_values: BTreeMap<Address, crate::theymx::Value>,
    ) {
        self.update_tx.send(OutputAgentUpdate::SetHighlightedValues(highlighted_values)).unwrap();
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

        thread::Builder::new()
            .name("output_agent".to_string())
            .spawn_with_priority(thread_priority::ThreadPriority::Max, move |prio_result| {
                assert!(prio_result.is_ok());

                let tick_interval = updater.tick_interval;
                let mut deadline = Instant::now() + tick_interval;

                loop {
                    let tick_start = Instant::now();

                    // Drain all pending updates that arrived since last tick.
                    let mut updates = Vec::<OutputAgentUpdate>::new();
                    loop {
                        match rx.try_recv() {
                            Ok(update) => updates.push(update),
                            Err(mpsc::TryRecvError::Empty) => break,
                            Err(mpsc::TryRecvError::Disconnected) => {
                                log::warn!(
                                    "update channel disconnected; continuing with local state"
                                );
                                break;
                            }
                        }
                    }

                    // Composite and transmit.
                    let output = updater.composite_pipeline(updates).clone();
                    updater.transmit_resolved_multiverse(output);

                    let tick_end = Instant::now();
                    let actual_tick_duration = tick_end.duration_since(tick_start);
                    let exec_overrun =
                        actual_tick_duration.checked_sub(tick_interval).unwrap_or(Duration::ZERO);

                    // Scheduling.
                    let now = Instant::now();
                    if now < deadline {
                        spin_sleep::sleep(deadline - now);
                        deadline += tick_interval;

                        log::trace!(
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
            })
            .expect("should spawn output agent thread");
    }
}

pub enum OutputAgentUpdate {
    SetAttributeValues(AttributeValues),
    SetHighlightedValues(BTreeMap<Address, crate::theymx::Value>),
}

struct Pipeline {
    base: Multiverse,
    updates: AttributeValues,
    highlights: BTreeMap<Address, crate::theymx::Value>,

    output: Multiverse,
}

impl Pipeline {
    pub fn new(default_multiverse: Multiverse) -> Self {
        Self {
            base: default_multiverse.clone(),
            updates: AttributeValues::new(),
            highlights: BTreeMap::new(),
            output: default_multiverse.clone(),
        }
    }

    pub fn composite(&mut self, project: &Project, updates: Vec<OutputAgentUpdate>) {
        for update in updates {
            match update {
                OutputAgentUpdate::SetAttributeValues(attribute_values) => {
                    self.updates.extend(attribute_values)
                }
                OutputAgentUpdate::SetHighlightedValues(highlighted_values) => {
                    self.highlights = highlighted_values.clone();
                }
            }
        }

        self.output = self.base.clone();

        resolver::resolve(&self.updates, &project.stage(), &mut self.output);

        for (address, value) in &self.highlights {
            self.output.set_value(address, *value);
        }
    }
}

struct Updater {
    project: Arc<Project>,

    tick_interval: Duration,
    output_tx: crossbeam_channel::Sender<Multiverse>,

    pipeline: RwLock<Pipeline>,
}

impl Updater {
    pub fn new(output_tx: crossbeam_channel::Sender<Multiverse>, project: Arc<Project>) -> Self {
        let default_multiverse = project.stage().default_multiverse().clone();
        let pipeline = RwLock::new(Pipeline::new(default_multiverse));

        Self { project, tick_interval: Duration::from_secs_f64(1.0 / 44.0), output_tx, pipeline }
    }

    fn composite_pipeline(&self, updates: Vec<OutputAgentUpdate>) -> Multiverse {
        let mut pipeline = self.pipeline.write().unwrap();
        pipeline.composite(&self.project, updates);
        pipeline.output.clone()
    }

    fn transmit_resolved_multiverse(&self, multiverse: Multiverse) {
        let _ = self.output_tx.send(multiverse);
    }
}
