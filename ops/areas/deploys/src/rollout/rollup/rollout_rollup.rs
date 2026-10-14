use super::*;

use acme_kit::text;

/// Where a rollout rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RolloutRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RolloutRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRollup {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub attempts: i64,
    pub external_ref: String,
    pub rate: f64,
    pub status: RolloutRollupStatus,
}

impl RolloutRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutRollupStatus) -> bool {
        match self.status {
            RolloutRollupStatus::Planned => {
                return next == RolloutRollupStatus::Completed;
            }
            RolloutRollupStatus::Scheduled => {
                match next {
                    RolloutRollupStatus::InProgress | RolloutRollupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutRollupStatus::InProgress => {
                match next {
                    RolloutRollupStatus::Completed | RolloutRollupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutRollupStatus::Completed => {
                match next {
                    RolloutRollupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RolloutRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.retries as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutRollup {
        RolloutRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            attempts: self.attempts,
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollout rollup in status planned.
pub fn new_rollout_rollup(code: &str, label: &str) -> RolloutRollup {
    RolloutRollup {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        attempts: 0,
        external_ref: "".to_string(),
        rate: 0.0,
        status: RolloutRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollout_rollup("code-963", "label-518");
        assert_eq!(item.status, RolloutRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_rollout_rollup("code-963", "label-518");
        if item.move_to(RolloutRollupStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutRollupStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollout_rollup("code-963", "label-518");
        item.status = RolloutRollupStatus::Cancelled;
        assert!(
            item.move_to(RolloutRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
