use super::*;

use acme_kit::text;

/// Where a strict canary milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictCanaryMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictCanaryMilestoneStatus {
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

impl std::fmt::Display for StrictCanaryMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict canary milestone as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryMilestone {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub tags: Vec<String>,
    pub ceiling: f64,
    pub sequence: i64,
    pub audience: String,
    pub priority: i64,
    pub threshold: f64,
    pub status: StrictCanaryMilestoneStatus,
}

impl StrictCanaryMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictCanaryMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictCanaryMilestoneStatus) -> bool {
        match self.status {
            StrictCanaryMilestoneStatus::Idle => {
                return next == StrictCanaryMilestoneStatus::Warming;
            }
            StrictCanaryMilestoneStatus::Warming => {
                return next == StrictCanaryMilestoneStatus::Draining;
            }
            StrictCanaryMilestoneStatus::Ready => {
                match next {
                    StrictCanaryMilestoneStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictCanaryMilestoneStatus::Draining => {
                return next == StrictCanaryMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictCanaryMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict canary milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        (self.ceiling * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictCanaryMilestone {
        StrictCanaryMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            tags: self.tags.clone(),
            ceiling: self.ceiling,
            sequence: self.sequence,
            audience: self.audience.clone(),
            priority: self.priority,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict canary milestone in status idle.
pub fn new_strict_canary_milestone(code: &str, name: &str) -> StrictCanaryMilestone {
    StrictCanaryMilestone {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        tags: Vec::new(),
        ceiling: 0.0,
        sequence: 0,
        audience: "".to_string(),
        priority: 0,
        threshold: 0.0,
        status: StrictCanaryMilestoneStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_canary_milestone("code-593", "name-414");
        assert_eq!(item.status, StrictCanaryMilestoneStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_strict_canary_milestone("code-593", "name-414");
        if item.move_to(StrictCanaryMilestoneStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictCanaryMilestoneStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_canary_milestone("code-593", "name-414");
        item.status = StrictCanaryMilestoneStatus::Stopped;
        assert!(
            item.move_to(StrictCanaryMilestoneStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
