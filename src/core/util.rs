use std::time::{Duration, Instant};

pub struct TimingLogger {
    start: Option<Instant>,
    durations: Vec<Duration>,
    interval: usize,
    iteration: usize,
    label: String,
}

impl TimingLogger {
    pub fn new(label: impl AsRef<str>, interval: usize) -> Self {
        TimingLogger {
            label: label.as_ref().to_string(),
            start: None,
            durations: Vec::new(),
            interval,
            iteration: 0,
        }
    }

    /// Call at the start of each loop iteration.
    pub fn record(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Call at the end of each loop iteration.
    pub fn stop(&mut self) {
        if let Some(s) = self.start.take() {
            let duration = s.elapsed();
            self.durations.push(duration);
        }

        self.iteration += 1;

        if self.iteration.is_multiple_of(self.interval) {
            self.log_timings();
            // Remove the oldest durations so that only the most recent `interval` are kept
            if self.durations.len() > self.interval {
                let remove_count = self.durations.len() - self.interval;
                self.durations.drain(0..remove_count);
            }
        }
    }

    pub fn average(&self) -> Option<Duration> {
        if self.durations.is_empty() {
            None
        } else {
            // Only average over the most recent `interval` durations
            let len = self.durations.len().min(self.interval);
            let total: Duration =
                self.durations[self.durations.len().saturating_sub(len)..].iter().sum();
            Some(total / len as u32)
        }
    }

    pub fn log_timings(&self) {
        let avg_str = if let Some(avg) = self.average() {
            format!("{:.2?}", avg)
        } else {
            "<no records>".to_string()
        };
        let avg_str_padded = format!("{:<20}", avg_str);
        log::debug!("{} timing avg (over {}) = {}", self.label, self.interval, avg_str_padded,);
    }
}
