use super::*;

use acme_kit::text;

/// Where a failover claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FailoverClaimStatus {
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

impl std::fmt::Display for FailoverClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverClaim {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub owner_id: String,
    pub source: String,
    pub status: FailoverClaimStatus,
}

impl FailoverClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverClaimStatus) -> bool {
        match self.status {
            FailoverClaimStatus::Open => {
                match next {
                    FailoverClaimStatus::Reopened | FailoverClaimStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverClaimStatus::InReview => {
                match next {
                    FailoverClaimStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverClaimStatus::Resolved => {
                match next {
                    FailoverClaimStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverClaimStatus::Reopened => {
                return next == FailoverClaimStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "failover claim", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> FailoverClaim {
        FailoverClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a failover claim in status open.
pub fn new_failover_claim(code: &str, label: &str) -> FailoverClaim {
    FailoverClaim {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        owner_id: "".to_string(),
        source: "".to_string(),
        status: FailoverClaimStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_failover_claim("code-219", "label-257");
        assert_eq!(item.status, FailoverClaimStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_failover_claim("code-219", "label-257");
        if item.move_to(FailoverClaimStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverClaimStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_failover_claim("code-219", "label-257");
        item.status = FailoverClaimStatus::Archived;
        assert!(
            item.move_to(FailoverClaimStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
