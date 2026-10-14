use super::*;

use acme_kit::text;

/// Where a probe cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ProbeCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ProbeCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One probe cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCycle {
    pub code: String,
    pub name: String,
    pub country: String,
    pub duration_minutes: i64,
    pub ttl_seconds: i64,
    pub quantity: i64,
    pub status: ProbeCycleStatus,
}

impl ProbeCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeCycleStatus) -> bool {
        match self.status {
            ProbeCycleStatus::Idle => {
                return next == ProbeCycleStatus::Warming || next == ProbeCycleStatus::Ready;
            }
            ProbeCycleStatus::Warming => {
                return next == ProbeCycleStatus::Ready || next == ProbeCycleStatus::Stopped;
            }
            ProbeCycleStatus::Ready => {
                return next == ProbeCycleStatus::Draining;
            }
            ProbeCycleStatus::Draining => {
                match next {
                    ProbeCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeCycle {
        ProbeCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            duration_minutes: self.duration_minutes,
            ttl_seconds: self.ttl_seconds,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a probe cycle in status idle.
pub fn new_probe_cycle(code: &str, name: &str) -> ProbeCycle {
    ProbeCycle {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        duration_minutes: 0,
        ttl_seconds: 0,
        quantity: 0,
        status: ProbeCycleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_probe_cycle("code-639", "name-704");
        assert_eq!(item.status, ProbeCycleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_probe_cycle("code-639", "name-704");
        if item.move_to(ProbeCycleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeCycleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_probe_cycle("code-639", "name-704");
        item.status = ProbeCycleStatus::Stopped;
        assert!(item.move_to(ProbeCycleStatus::Idle).is_err(), "expected move to fail");
    }
}
