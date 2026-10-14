use super::*;

use acme_kit::text;

/// Where a seasonal rollout cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRolloutCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalRolloutCycleStatus {
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

impl std::fmt::Display for SeasonalRolloutCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal rollout cycle as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutCycle {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub summary: String,
    pub notes: String,
    pub active: bool,
    pub quantity: i64,
    pub status: SeasonalRolloutCycleStatus,
}

impl SeasonalRolloutCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRolloutCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRolloutCycleStatus) -> bool {
        match self.status {
            SeasonalRolloutCycleStatus::Idle => {
                return next == SeasonalRolloutCycleStatus::Warming || next == SeasonalRolloutCycleStatus::Stopped;
            }
            SeasonalRolloutCycleStatus::Warming => {
                return next == SeasonalRolloutCycleStatus::Ready;
            }
            SeasonalRolloutCycleStatus::Ready => {
                return next == SeasonalRolloutCycleStatus::Draining || next == SeasonalRolloutCycleStatus::Stopped;
            }
            SeasonalRolloutCycleStatus::Draining => {
                match next {
                    SeasonalRolloutCycleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalRolloutCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal rollout cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> SeasonalRolloutCycle {
        SeasonalRolloutCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            summary: self.summary.clone(),
            notes: self.notes.clone(),
            active: self.active,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal rollout cycle in status idle.
pub fn new_seasonal_rollout_cycle(code: &str, label: &str) -> SeasonalRolloutCycle {
    SeasonalRolloutCycle {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        summary: "".to_string(),
        notes: "".to_string(),
        active: false,
        quantity: 0,
        status: SeasonalRolloutCycleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_rollout_cycle("code-384", "label-308");
        assert_eq!(item.status, SeasonalRolloutCycleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_seasonal_rollout_cycle("code-384", "label-308");
        if item.move_to(SeasonalRolloutCycleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRolloutCycleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_rollout_cycle("code-384", "label-308");
        item.status = SeasonalRolloutCycleStatus::Stopped;
        assert!(
            item.move_to(SeasonalRolloutCycleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
