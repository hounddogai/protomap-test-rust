use super::*;

use acme_kit::text;

/// Where a manual verifier budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualVerifierBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualVerifierBudgetStatus {
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

impl std::fmt::Display for ManualVerifierBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual verifier budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierBudget {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub capacity: i64,
    pub score: f64,
    pub burst: i64,
    pub segment: String,
    pub status: ManualVerifierBudgetStatus,
}

impl ManualVerifierBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualVerifierBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualVerifierBudgetStatus) -> bool {
        match self.status {
            ManualVerifierBudgetStatus::Planned => {
                match next {
                    ManualVerifierBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierBudgetStatus::Scheduled => {
                return next == ManualVerifierBudgetStatus::InProgress || next == ManualVerifierBudgetStatus::Completed;
            }
            ManualVerifierBudgetStatus::InProgress => {
                match next {
                    ManualVerifierBudgetStatus::Completed | ManualVerifierBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierBudgetStatus::Completed => {
                return next == ManualVerifierBudgetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualVerifierBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual verifier budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.capacity as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualVerifierBudget {
        ManualVerifierBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            capacity: self.capacity,
            score: self.score,
            burst: self.burst,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual verifier budget in status planned.
pub fn new_manual_verifier_budget(code: &str, title: &str) -> ManualVerifierBudget {
    ManualVerifierBudget {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        capacity: 0,
        score: 0.0,
        burst: 0,
        segment: "".to_string(),
        status: ManualVerifierBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_verifier_budget("code-709", "title-800");
        assert_eq!(item.status, ManualVerifierBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_manual_verifier_budget("code-709", "title-800");
        if item.move_to(ManualVerifierBudgetStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualVerifierBudgetStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_verifier_budget("code-709", "title-800");
        item.status = ManualVerifierBudgetStatus::Cancelled;
        assert!(
            item.move_to(ManualVerifierBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
