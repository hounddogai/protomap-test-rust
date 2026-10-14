use super::*;

use acme_kit::text;

/// Where a rolling flow log budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingFlowLogBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingFlowLogBudgetStatus {
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

impl std::fmt::Display for RollingFlowLogBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling flow log budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogBudget {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub revision: i64,
    pub burst: i64,
    pub active: bool,
    pub checksum: String,
    pub channel: String,
    pub amount: f64,
    pub summary: String,
    pub audience: String,
    pub status: RollingFlowLogBudgetStatus,
}

impl RollingFlowLogBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingFlowLogBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingFlowLogBudgetStatus) -> bool {
        match self.status {
            RollingFlowLogBudgetStatus::Open => {
                return next == RollingFlowLogBudgetStatus::Resolved;
            }
            RollingFlowLogBudgetStatus::InReview => {
                return next == RollingFlowLogBudgetStatus::Resolved || next == RollingFlowLogBudgetStatus::Reopened;
            }
            RollingFlowLogBudgetStatus::Resolved => {
                match next {
                    RollingFlowLogBudgetStatus::Reopened | RollingFlowLogBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFlowLogBudgetStatus::Reopened => {
                match next {
                    RollingFlowLogBudgetStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingFlowLogBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling flow log budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.burst as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingFlowLogBudget {
        RollingFlowLogBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            revision: self.revision,
            burst: self.burst,
            active: self.active,
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            amount: self.amount,
            summary: self.summary.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rolling flow log budget in status open.
pub fn new_rolling_flow_log_budget(code: &str, title: &str) -> RollingFlowLogBudget {
    RollingFlowLogBudget {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        revision: 0,
        burst: 0,
        active: false,
        checksum: "".to_string(),
        channel: "".to_string(),
        amount: 0.0,
        summary: "".to_string(),
        audience: "".to_string(),
        status: RollingFlowLogBudgetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_flow_log_budget("code-436", "title-41");
        assert_eq!(item.status, RollingFlowLogBudgetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rolling_flow_log_budget("code-436", "title-41");
        if item.move_to(RollingFlowLogBudgetStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingFlowLogBudgetStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_flow_log_budget("code-436", "title-41");
        item.status = RollingFlowLogBudgetStatus::Archived;
        assert!(
            item.move_to(RollingFlowLogBudgetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
