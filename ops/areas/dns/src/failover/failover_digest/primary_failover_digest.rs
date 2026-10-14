use super::*;

use acme_kit::text;

/// Where a primary failover digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFailoverDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryFailoverDigestStatus {
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

impl std::fmt::Display for PrimaryFailoverDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary failover digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverDigest {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub attempts: i64,
    pub burst: i64,
    pub status: PrimaryFailoverDigestStatus,
}

impl PrimaryFailoverDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFailoverDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFailoverDigestStatus) -> bool {
        match self.status {
            PrimaryFailoverDigestStatus::Open => {
                match next {
                    PrimaryFailoverDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverDigestStatus::InReview => {
                return next == PrimaryFailoverDigestStatus::Resolved || next == PrimaryFailoverDigestStatus::Reopened;
            }
            PrimaryFailoverDigestStatus::Resolved => {
                match next {
                    PrimaryFailoverDigestStatus::Reopened | PrimaryFailoverDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverDigestStatus::Reopened => {
                match next {
                    PrimaryFailoverDigestStatus::Archived => {
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

    pub fn move_to(&mut self, next: PrimaryFailoverDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary failover digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary failover digest", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.burst as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryFailoverDigest {
        PrimaryFailoverDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            attempts: self.attempts,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary failover digest in status open.
pub fn new_primary_failover_digest(code: &str, title: &str) -> PrimaryFailoverDigest {
    PrimaryFailoverDigest {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        attempts: 0,
        burst: 0,
        status: PrimaryFailoverDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_failover_digest("code-623", "title-762");
        assert_eq!(item.status, PrimaryFailoverDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_primary_failover_digest("code-623", "title-762");
        if item.move_to(PrimaryFailoverDigestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryFailoverDigestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_failover_digest("code-623", "title-762");
        item.status = PrimaryFailoverDigestStatus::Archived;
        assert!(
            item.move_to(PrimaryFailoverDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
