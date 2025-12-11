use std::sync::Arc;
use std::time::Duration;

use tokio::task;

use crate::client::Client;
use crate::core::util::TimingLogger;
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
    /// The current frame number.
    pub frame: usize,
    /// Reference to the baked patch describing the fixture configuration.
    pub patch: &'bp BakedPatch,
    /// Mutable reference to the attribute values to be set for this frame.
    pub values: &'val mut AttributeValues,
}
