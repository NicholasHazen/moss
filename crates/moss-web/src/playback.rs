//! Wall time requests complete ticks. The simulation itself never reads this clock.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

const MAX_TICKS_PER_FRAME: u32 = 4;

#[derive(Default)]
pub struct Playback {
    pub running: bool,
    pub slowed: bool,
    pub suspended: bool,
    last_ms: Option<f64>,
    accumulated_seconds: f64,
    hidden_epoch: u32,
}

impl Playback {
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
        self.last_ms = None;
        self.accumulated_seconds = 0.0;
        self.slowed = false;
        if running {
            self.suspended = false;
        }
    }

    pub fn visibility(&mut self, hidden: bool, epoch: u32) {
        // The epoch catches hide+show even if no render frame ran while hidden.
        if hidden || epoch != self.hidden_epoch {
            self.set_running(false);
            self.suspended = true;
        }
        self.hidden_epoch = epoch;
    }

    pub fn advance(&mut self, now_ms: f64) -> u32 {
        let previous = self.last_ms.replace(now_ms);
        if !self.running {
            return 0;
        }
        let elapsed = previous.map_or(0.0, |last| ((now_ms - last) / 1000.0).max(0.0));
        let tick_seconds = f64::from(moss_sim::TICK_SECONDS);
        let budget = tick_seconds * f64::from(MAX_TICKS_PER_FRAME);
        self.slowed = elapsed > budget;
        // Discard excess wall time under load; never build a hidden catch-up queue.
        self.accumulated_seconds += elapsed.min(budget);
        let ticks = (self.accumulated_seconds / tick_seconds).floor() as u32;
        self.accumulated_seconds -= f64::from(ticks) * tick_seconds;
        ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_resume_and_hidden_return_discard_backlog() {
        let mut playback = Playback::default();
        assert_eq!(playback.advance(0.0), 0);
        playback.set_running(true);
        assert_eq!(playback.advance(0.0), 0);
        assert_eq!(playback.advance(250.0), 1);
        playback.visibility(false, 1); // hidden and shown between frames
        assert!(!playback.running);
        assert_eq!(playback.advance(60_000.0), 0);
        playback.set_running(true);
        assert_eq!(playback.advance(60_000.0), 0);
        assert_eq!(playback.advance(60_250.0), 1);
    }

    #[test]
    fn render_frequency_does_not_change_requested_ticks() {
        fn count(frame_ms: f64) -> u32 {
            let mut playback = Playback::default();
            playback.set_running(true);
            playback.advance(0.0);
            (1..=(2000.0 / frame_ms) as u32)
                .map(|frame| playback.advance(f64::from(frame) * frame_ms))
                .sum()
        }
        assert_eq!(count(125.0), 8);
        assert_eq!(count(250.0), 8);
    }

    #[test]
    fn slow_frames_are_bounded_and_reported() {
        let mut playback = Playback::default();
        playback.set_running(true);
        playback.advance(0.0);
        assert_eq!(playback.advance(60_000.0), MAX_TICKS_PER_FRAME);
        assert!(playback.slowed);
        assert_eq!(playback.advance(60_250.0), 1);
    }
}
