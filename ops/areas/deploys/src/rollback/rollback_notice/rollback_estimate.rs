use super::*;

use acme_kit::text;

/// Where a rollback estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollbackEstimateStatus {
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

impl std::fmt::Display for RollbackEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollback estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEstimate {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub archived: bool,
    pub cluster: String,
    pub active: bool,
    pub owner_id: String,
    pub attempts: i64,
    pub locale: String,
    pub budget: f64,
    pub status: RollbackEstimateStatus,
}

impl RollbackEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackEstimateStatus) -> bool {
        match self.status {
            RollbackEstimateStatus::Planned => {
                return next == RollbackEstimateStatus::Scheduled || next == RollbackEstimateStatus::InProgress;
            }
            RollbackEstimateStatus::Scheduled => {
                match next {
                    RollbackEstimateStatus::InProgress | RollbackEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackEstimateStatus::InProgress => {
                match next {
                    RollbackEstimateStatus::Completed | RollbackEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackEstimateStatus::Completed => {
                return next == RollbackEstimateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback estimate", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.attempts as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackEstimate {
        RollbackEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            archived: self.archived,
            cluster: self.cluster.clone(),
            active: self.active,
            owner_id: self.owner_id.clone(),
            attempts: self.attempts,
            locale: self.locale.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rollback estimate in status planned.
pub fn new_rollback_estimate(code: &str, label: &str) -> RollbackEstimate {
    RollbackEstimate {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        archived: false,
        cluster: "".to_string(),
        active: false,
        owner_id: "".to_string(),
        attempts: 0,
        locale: "".to_string(),
        budget: 0.0,
        status: RollbackEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollback_estimate("code-977", "label-511");
        assert_eq!(item.status, RollbackEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rollback_estimate("code-977", "label-511");
        if item.move_to(RollbackEstimateStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackEstimateStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollback_estimate("code-977", "label-511");
        item.status = RollbackEstimateStatus::Cancelled;
        assert!(
            item.move_to(RollbackEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
