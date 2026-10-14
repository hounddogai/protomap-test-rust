use super::*;

use acme_kit::text;

/// Where a rollback variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollbackVarianceStatus {
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

impl std::fmt::Display for RollbackVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollback variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackVariance {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub priority: i64,
    pub unit: String,
    pub status: RollbackVarianceStatus,
}

impl RollbackVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackVarianceStatus) -> bool {
        match self.status {
            RollbackVarianceStatus::Planned => {
                return next == RollbackVarianceStatus::InProgress;
            }
            RollbackVarianceStatus::Scheduled => {
                return next == RollbackVarianceStatus::InProgress || next == RollbackVarianceStatus::Completed;
            }
            RollbackVarianceStatus::InProgress => {
                match next {
                    RollbackVarianceStatus::Completed | RollbackVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackVarianceStatus::Completed => {
                match next {
                    RollbackVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollbackVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.burst as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackVariance {
        RollbackVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            priority: self.priority,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rollback variance in status planned.
pub fn new_rollback_variance(code: &str, title: &str) -> RollbackVariance {
    RollbackVariance {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        priority: 0,
        unit: "".to_string(),
        status: RollbackVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollback_variance("code-902", "title-924");
        assert_eq!(item.status, RollbackVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_rollback_variance("code-902", "title-924");
        if item.move_to(RollbackVarianceStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackVarianceStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollback_variance("code-902", "title-924");
        item.status = RollbackVarianceStatus::Cancelled;
        assert!(
            item.move_to(RollbackVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
