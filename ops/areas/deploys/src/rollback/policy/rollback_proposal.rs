use super::*;

use acme_kit::text;

/// Where a rollback proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollbackProposalStatus {
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

impl std::fmt::Display for RollbackProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback proposal as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProposal {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub phone: String,
    pub external_ref: String,
    pub currency: String,
    pub duration_minutes: i64,
    pub status: RollbackProposalStatus,
}

impl RollbackProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackProposalStatus) -> bool {
        match self.status {
            RollbackProposalStatus::Open => {
                return next == RollbackProposalStatus::Archived;
            }
            RollbackProposalStatus::InReview => {
                match next {
                    RollbackProposalStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackProposalStatus::Resolved => {
                return next == RollbackProposalStatus::Reopened || next == RollbackProposalStatus::Archived;
            }
            RollbackProposalStatus::Reopened => {
                return next == RollbackProposalStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollback proposal", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackProposal {
        RollbackProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            phone: self.phone.clone(),
            external_ref: self.external_ref.clone(),
            currency: self.currency.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rollback proposal in status open.
pub fn new_rollback_proposal(code: &str, title: &str) -> RollbackProposal {
    RollbackProposal {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        owner_id: "".to_string(),
        phone: "".to_string(),
        external_ref: "".to_string(),
        currency: "".to_string(),
        duration_minutes: 0,
        status: RollbackProposalStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollback_proposal("code-129", "title-895");
        assert_eq!(item.status, RollbackProposalStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_rollback_proposal("code-129", "title-895");
        if item.move_to(RollbackProposalStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackProposalStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollback_proposal("code-129", "title-895");
        item.status = RollbackProposalStatus::Archived;
        assert!(
            item.move_to(RollbackProposalStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
