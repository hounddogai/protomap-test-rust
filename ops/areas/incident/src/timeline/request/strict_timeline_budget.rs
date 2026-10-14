use super::*;

use acme_kit::text;

/// Where a strict timeline budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictTimelineBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictTimelineBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for StrictTimelineBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict timeline budget as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineBudget {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub discount: f64,
    pub burst: i64,
    pub port: i64,
    pub status: StrictTimelineBudgetStatus,
}

impl StrictTimelineBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictTimelineBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictTimelineBudgetStatus) -> bool {
        match self.status {
            StrictTimelineBudgetStatus::Open => {
                return next == StrictTimelineBudgetStatus::Resolved;
            }
            StrictTimelineBudgetStatus::InReview => {
                return next == StrictTimelineBudgetStatus::Resolved || next == StrictTimelineBudgetStatus::Reopened;
            }
            StrictTimelineBudgetStatus::Resolved => {
                return next == StrictTimelineBudgetStatus::Reopened;
            }
            StrictTimelineBudgetStatus::Reopened => {
                match next {
                    StrictTimelineBudgetStatus::Archived => {
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

    pub fn move_to(&mut self, next: StrictTimelineBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict timeline budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.discount * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictTimelineBudget {
        StrictTimelineBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            discount: self.discount,
            burst: self.burst,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict timeline budget in status open.
pub fn new_strict_timeline_budget(code: &str, label: &str) -> StrictTimelineBudget {
    StrictTimelineBudget {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        discount: 0.0,
        burst: 0,
        port: 0,
        status: StrictTimelineBudgetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_timeline_budget("code-474", "label-359");
        assert_eq!(item.status, StrictTimelineBudgetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_strict_timeline_budget("code-474", "label-359");
        if item.move_to(StrictTimelineBudgetStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictTimelineBudgetStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_timeline_budget("code-474", "label-359");
        item.status = StrictTimelineBudgetStatus::Archived;
        assert!(
            item.move_to(StrictTimelineBudgetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
