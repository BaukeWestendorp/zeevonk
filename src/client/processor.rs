use std::sync::Arc;
use std::time::Duration;

use tokio::task;

use crate::client::Client;
use crate::core::gdcs::FixturePath;
use crate::core::util::TimingLogger;
use crate::prelude::{Attribute, ClampedValue};
use crate::server::{AttributeValues, BakedPatch};

impl Client {
    /// Registers a processor closure that will run in a background task.
    ///
    /// The processor is invoked on a fixed 25ms interval (i.e. 40Hz).
    ///
    /// The populated attribute values are sent to the server on each frame.
    pub async fn register_processor<F: Fn(ProcessorContext) + Send + Sync + 'static>(
        &self,
        processor: F,
    ) {
        let inner = Arc::clone(&self.inner);
        let processor = Arc::new(processor);
        task::spawn(async move {
            let baked_patch = match inner.lock().await.request_patch().await {
                Ok(p) => p,
                Err(err) => {
                    log::error!("could not get baked patch for processor: {err}");
                    return;
                }
            };

            // Use a fixed interval starting one period from now to get accurate 33ms ticks.
            let period = Duration::from_millis(33);
            let start = tokio::time::Instant::now() + period;
            let mut interval = tokio::time::interval_at(start, period);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            let mut timing_logger = TimingLogger::new("processor", period.as_millis() as usize * 5);

            let mut frame = 0;
            loop {
                // Wait until the next scheduled tick. Using interval_at fixes the schedule
                // to the chosen start instant and period, minimizing drift.
                let scheduled_instant = interval.tick().await;
                let start_instant = tokio::time::Instant::now();

                // How late we are relative to the scheduled instant.
                let lateness =
                    start_instant.checked_duration_since(scheduled_instant).unwrap_or_default();

                if lateness > period {
                    log::warn!(
                        "processor is running behind schedule by {:.2?} (frame {})",
                        lateness,
                        frame
                    );
                } else {
                    log::trace!(
                        "processor is running behind schedule by {:.2?} (frame {})",
                        lateness,
                        frame
                    );
                }

                timing_logger.record();

                let mut values = AttributeValues::new();
                let cx = ProcessorContext { frame, patch: &baked_patch, values: &mut values };
                (processor.as_ref())(cx);

                // Await the result to ensure the request is sent and handled.
                let send_result = inner.lock().await.request_set_attribute_values(values).await;

                timing_logger.stop();

                if let Err(err) = send_result {
                    log::error!("failed to send attribute values: {err}");
                    break;
                }

                frame += 1;
            }
        })
        .await
        .unwrap();
    }
}

/// Context passed to the processor closure for each frame.
pub struct ProcessorContext<'bp, 'val> {
    frame: usize,
    patch: &'bp BakedPatch,
    values: &'val mut AttributeValues,
}

impl ProcessorContext<'_, '_> {
    /// The current frame number.
    pub fn frame(&self) -> usize {
        self.frame
    }

    /// Reference to the baked patch describing the fixture configuration.
    pub fn patch(&self) -> &BakedPatch {
        self.patch
    }

    /// Mutable reference to the attribute values to be set for this frame.
    pub fn values_mut(&mut self) -> &mut AttributeValues {
        self.values
    }

    pub fn set_attribute(
        &mut self,
        fixture_collection: impl Into<FixtureCollection>,
        attribute: Attribute,
        value: impl Into<ClampedValue>,
        include_children: bool,
    ) {
        let value = value.into();
        for path in fixture_collection.into().paths() {
            if include_children {
                let paths = self
                    .patch()
                    .fixtures()
                    .keys()
                    .filter(|p| p.contains(path))
                    .copied()
                    .collect::<Vec<_>>();

                for p in paths {
                    self.values_mut().set(p, attribute, value);
                }
            } else {
                self.values_mut().set(*path, attribute, value);
            }
        }
    }
}

pub struct FixtureCollection(Vec<FixturePath>);

impl FixtureCollection {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, fixture_path: FixturePath) {
        self.0.push(fixture_path)
    }

    pub fn paths(&self) -> &[FixturePath] {
        &self.0
    }
}

impl From<FixturePath> for FixtureCollection {
    fn from(fixture_path: FixturePath) -> Self {
        Self(vec![fixture_path])
    }
}

impl From<Vec<FixturePath>> for FixtureCollection {
    fn from(fixture_paths: Vec<FixturePath>) -> Self {
        Self(fixture_paths)
    }
}

impl From<&[FixturePath]> for FixtureCollection {
    fn from(fixture_paths: &[FixturePath]) -> Self {
        Self(fixture_paths.to_vec())
    }
}

impl<const N: usize> From<[FixturePath; N]> for FixtureCollection {
    fn from(fixture_paths: [FixturePath; N]) -> Self {
        Self(fixture_paths.to_vec())
    }
}
