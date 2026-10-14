use super::*;

use acme_kit::text;

/// Where a rollout ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RolloutLedgerStatus {
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

impl std::fmt::Display for RolloutLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollout ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutLedger {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub limit: i64,
    pub margin: f64,
    pub budget: f64,
    pub revision: i64,
    pub starts_at: i64,
    pub status: RolloutLedgerStatus,
}

impl RolloutLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutLedgerStatus) -> bool {
        match self.status {
            RolloutLedgerStatus::Open => {
                match next {
                    RolloutLedgerStatus::InReview | RolloutLedgerStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutLedgerStatus::InReview => {
                match next {
                    RolloutLedgerStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutLedgerStatus::Resolved => {
                return next == RolloutLedgerStatus::Archived;
            }
            RolloutLedgerStatus::Reopened => {
                return next == RolloutLedgerStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout ledger", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.minimum - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutLedger {
        RolloutLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            limit: self.limit,
            margin: self.margin,
            budget: self.budget,
            revision: self.revision,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rollout ledger in status open.
pub fn new_rollout_ledger(code: &str, title: &str) -> RolloutLedger {
    RolloutLedger {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        limit: 0,
        margin: 0.0,
        budget: 0.0,
        revision: 0,
        starts_at: 0,
        status: RolloutLedgerStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollout_ledger("code-129", "title-553");
        assert_eq!(item.status, RolloutLedgerStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_rollout_ledger("code-129", "title-553");
        if item.move_to(RolloutLedgerStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutLedgerStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollout_ledger("code-129", "title-553");
        item.status = RolloutLedgerStatus::Archived;
        assert!(
            item.move_to(RolloutLedgerStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
