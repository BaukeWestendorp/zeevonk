use std::sync::Arc;
use std::time::Duration;

use tokio::task;

use crate::attr::Attribute;
use crate::client::Client;
use crate::packet::AttributeValues;
use crate::state::State;
use crate::state::fixture::FixturePath;
use crate::value::ClampedValue;

impl Client {
    pub async fn register_processor<F: Fn(ProcessorContext) + Send + Sync + 'static>(
        &self,
        processor: F,
    ) {
        let inner = Arc::clone(&self.inner);
        let processor = Arc::new(processor);
        task::spawn(async move {
            let state = match inner.lock().await.request_state().await {
                Ok(p) => p,
                Err(err) => {
                    log::error!("could not get show data for processor: {err}");
                    return;
                }
            };

            // Use a fixed interval starting one period from now to get accurate 33ms ticks.
            let period = Duration::from_millis(33);
            let start = tokio::time::Instant::now() + period;
            let mut interval = tokio::time::interval_at(start, period);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);

            let mut frame = 0;
            loop {
                // Wait until the next scheduled tick. Using interval_at fixes the schedule
                // to the chosen start instant and period, minimizing drift.
                interval.tick().await;

                let mut values = AttributeValues::new();
                let cx = ProcessorContext { frame, state: &state, values: &mut values };
                (processor.as_ref())(cx);

                // Await the result to ensure the request is sent and handled.
                let send_result = inner.lock().await.request_set_attribute_values(values).await;

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

pub struct ProcessorContext<'state, 'val> {
    frame: usize,
    state: &'state State,
    values: &'val mut AttributeValues,
}

impl ProcessorContext<'_, '_> {
    pub fn frame(&self) -> usize {
        self.frame
    }

    pub fn state(&self) -> &State {
        self.state
    }

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
                    .state()
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
