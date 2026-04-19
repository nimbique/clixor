use std::{
    num::NonZeroU64,
    time::{Duration, Instant},
};

pub struct ButtonState {
    pub enabled: bool,
    hold_start: Option<Instant>,
    last_click: Option<Instant>,
    toggle_prev: bool,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            enabled: true,
            hold_start: None,
            last_click: None,
            toggle_prev: false,
        }
    }
}

impl ButtonState {
    pub fn process_toggle(&mut self, key_held: bool) -> bool {
        let rising_edge = key_held && !self.toggle_prev;
        self.toggle_prev = key_held;
        if !rising_edge {
            return false;
        }
        self.enabled = !self.enabled;
        if !self.enabled {
            self.reset_timing();
        }
        true
    }

    pub fn tick(&mut self, phys_held: bool, params: &ClickParams) -> bool {
        if !self.enabled || !phys_held {
            self.reset_timing();
            return false;
        }
        let hold_start = *self.hold_start.get_or_insert_with(Instant::now);
        let interval = params.interval_at(hold_start.elapsed());
        let due = self.last_click.map_or(true, |t| t.elapsed() >= interval);
        if due {
            self.last_click = Some(Instant::now());
        }
        due
    }

    pub fn time_until_next(&self, params: &ClickParams) -> Duration {
        if !self.enabled {
            return Duration::MAX;
        }
        let Some(hold_start) = self.hold_start else {
            return Duration::MAX;
        };
        let interval = params.interval_at(hold_start.elapsed());
        self.last_click
            .map_or(Duration::ZERO, |t| interval.saturating_sub(t.elapsed()))
    }

    fn reset_timing(&mut self) {
        self.hold_start = None;
        self.last_click = None;
    }
}

#[derive(Clone, Copy)]
pub struct ClickParams {
    target_cps: f64,
    ramp_start: f64,
    ramp_duration: f64,
}

impl ClickParams {
    pub fn new(target_cps: f64, start_ratio: f64, ramp_duration_ms: NonZeroU64) -> Self {
        Self {
            target_cps,
            ramp_start: target_cps * start_ratio,
            ramp_duration: ramp_duration_ms.get() as f64 * 1e-3,
        }
    }

    #[inline]
    fn ramp_cps(&self, elapsed: Duration) -> f64 {
        let t = (elapsed.as_secs_f64() / self.ramp_duration).min(1.0);
        self.ramp_start + (self.target_cps - self.ramp_start) * t
    }

    #[inline]
    fn interval_at(&self, elapsed: Duration) -> Duration {
        Duration::from_secs_f64(1.0 / self.ramp_cps(elapsed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(cps: f64) -> ClickParams {
        ClickParams::new(cps, 0.2, NonZeroU64::new(500).unwrap())
    }

    #[test]
    fn ramp_starts_at_ratio() {
        let p = params(100.0);
        assert!((p.ramp_cps(Duration::ZERO) - 20.0).abs() < 1e-9);
    }

    #[test]
    fn ramp_ends_at_target() {
        let p = params(100.0);
        assert!((p.ramp_cps(Duration::from_secs(1)) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn ramp_clamps_above_duration() {
        let p = params(100.0);
        assert!((p.ramp_cps(Duration::from_secs(60)) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn toggle_fires_on_rising_edge_only() {
        let mut b = ButtonState::default();
        assert!(b.process_toggle(true));
        assert!(!b.process_toggle(true));
        assert!(!b.process_toggle(false));
        assert!(b.process_toggle(true));
    }

    #[test]
    fn toggle_disables_then_reenables() {
        let mut b = ButtonState::default();
        assert!(b.enabled);
        b.process_toggle(true);
        assert!(!b.enabled);
        b.process_toggle(false);
        b.process_toggle(true);
        assert!(b.enabled);
    }
}
