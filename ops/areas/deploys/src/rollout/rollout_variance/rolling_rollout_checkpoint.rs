use super::*;

use acme_kit::text;

/// Where a rolling rollout checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRolloutCheckpointStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingRolloutCheckpointStatus {
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

impl std::fmt::Display for RollingRolloutCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling rollout checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutCheckpoint {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub attempts: i64,
    pub namespace: String,
    pub status: RollingRolloutCheckpointStatus,
}

impl RollingRolloutCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRolloutCheckpointStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRolloutCheckpointStatus) -> bool {
        match self.status {
            RollingRolloutCheckpointStatus::Idle => {
                match next {
                    RollingRolloutCheckpointStatus::Warming | RollingRolloutCheckpointStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRolloutCheckpointStatus::Warming => {
                match next {
                    RollingRolloutCheckpointStatus::Draining | RollingRolloutCheckpointStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRolloutCheckpointStatus::Ready => {
                return next == RollingRolloutCheckpointStatus::Draining
                    || next == RollingRolloutCheckpointStatus::Stopped;
            }
            RollingRolloutCheckpointStatus::Draining => {
                return next == RollingRolloutCheckpointStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingRolloutCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling rollout checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.margin - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRolloutCheckpoint {
        RollingRolloutCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            attempts: self.attempts,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling rollout checkpoint in status idle.
pub fn new_rolling_rollout_checkpoint(code: &str, label: &str) -> RollingRolloutCheckpoint {
    RollingRolloutCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        attempts: 0,
        namespace: "".to_string(),
        status: RollingRolloutCheckpointStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_rollout_checkpoint("code-260", "label-525");
        assert_eq!(item.status, RollingRolloutCheckpointStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_rolling_rollout_checkpoint("code-260", "label-525");
        if item.move_to(RollingRolloutCheckpointStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRolloutCheckpointStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_rollout_checkpoint("code-260", "label-525");
        item.status = RollingRolloutCheckpointStatus::Stopped;
        assert!(
            item.move_to(RollingRolloutCheckpointStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
