use std::time::Duration;

pub struct TimingLogger {
    avg_window: usize,
    lateness_sum: Duration,
    proc_sum: Duration,
    send_sum: Duration,
    total_sum: Duration,
    frame_count: usize,
    over_period_count: usize,
}

impl TimingLogger {
    pub fn new(avg_window: usize) -> Self {
        Self {
            avg_window,
            lateness_sum: Duration::ZERO,
            proc_sum: Duration::ZERO,
            send_sum: Duration::ZERO,
            total_sum: Duration::ZERO,
            frame_count: 0,
            over_period_count: 0,
        }
    }

    pub fn record_frame(
        &mut self,
        lateness: Duration,
        proc_duration: Duration,
        send_duration: Duration,
        total_frame: Duration,
        period: Duration,
    ) {
        self.lateness_sum += lateness;
        self.proc_sum += proc_duration;
        self.send_sum += send_duration;
        self.total_sum += total_frame;
        self.frame_count += 1;
        if total_frame > period {
            self.over_period_count += 1;
        }

        // Log averages every avg_window frames
        if self.frame_count % self.avg_window == 0 {
            let avg_lateness = self.lateness_sum / self.avg_window as u32;
            let avg_proc = self.proc_sum / self.avg_window as u32;
            let avg_send = self.send_sum / self.avg_window as u32;

            log::debug!(
                "processor frame timing (avg over {:.2}): lateness={:.2?} proc={:.2?} send={:.2?}",
                self.avg_window,
                avg_lateness,
                avg_proc,
                avg_send,
            );
            self.lateness_sum = Duration::ZERO;
            self.proc_sum = Duration::ZERO;
            self.send_sum = Duration::ZERO;
        }

        // Warn if frame took longer than the period.
        if total_frame > period {
            log::warn!(
                "processor frame took {:.2?}, which exceeds the frame period of {:.2?}",
                total_frame,
                period
            );
        }
    }
}
