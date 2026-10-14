use super::*;

use acme_kit::text;

/// Where a rollback claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollbackClaimStatus {
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

impl std::fmt::Display for RollbackClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback claim as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackClaim {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub locale: String,
    pub budget: f64,
    pub status: RollbackClaimStatus,
}

impl RollbackClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackClaimStatus) -> bool {
        match self.status {
            RollbackClaimStatus::Open => {
                match next {
                    RollbackClaimStatus::Resolved | RollbackClaimStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackClaimStatus::InReview => {
                return next == RollbackClaimStatus::Resolved || next == RollbackClaimStatus::Reopened;
            }
            RollbackClaimStatus::Resolved => {
                match next {
                    RollbackClaimStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackClaimStatus::Reopened => {
                match next {
                    RollbackClaimStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollbackClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.score - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackClaim {
        RollbackClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            locale: self.locale.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rollback claim in status open.
pub fn new_rollback_claim(code: &str, title: &str) -> RollbackClaim {
    RollbackClaim {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        locale: "".to_string(),
        budget: 0.0,
        status: RollbackClaimStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollback_claim("code-727", "title-235");
        assert_eq!(item.status, RollbackClaimStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rollback_claim("code-727", "title-235");
        if item.move_to(RollbackClaimStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackClaimStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollback_claim("code-727", "title-235");
        item.status = RollbackClaimStatus::Archived;
        assert!(
            item.move_to(RollbackClaimStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
