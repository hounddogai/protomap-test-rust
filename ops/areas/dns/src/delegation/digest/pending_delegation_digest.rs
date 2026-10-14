use super::*;

use acme_kit::text;

/// Where a pending delegation digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDelegationDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingDelegationDigestStatus {
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

impl std::fmt::Display for PendingDelegationDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending delegation digest as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationDigest {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub updated_at: i64,
    pub rate: f64,
    pub principal: String,
    pub cipher: String,
    pub duration_minutes: i64,
    pub amount: f64,
    pub burst: i64,
    pub margin: f64,
    pub status: PendingDelegationDigestStatus,
}

impl PendingDelegationDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDelegationDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDelegationDigestStatus) -> bool {
        match self.status {
            PendingDelegationDigestStatus::Open => {
                match next {
                    PendingDelegationDigestStatus::InReview | PendingDelegationDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDelegationDigestStatus::InReview => {
                match next {
                    PendingDelegationDigestStatus::Resolved | PendingDelegationDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDelegationDigestStatus::Resolved => {
                return next == PendingDelegationDigestStatus::Reopened
                    || next == PendingDelegationDigestStatus::Archived;
            }
            PendingDelegationDigestStatus::Reopened => {
                match next {
                    PendingDelegationDigestStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingDelegationDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending delegation digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.grace_days as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDelegationDigest {
        PendingDelegationDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            updated_at: self.updated_at,
            rate: self.rate,
            principal: self.principal.clone(),
            cipher: self.cipher.clone(),
            duration_minutes: self.duration_minutes,
            amount: self.amount,
            burst: self.burst,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending delegation digest in status open.
pub fn new_pending_delegation_digest(code: &str, name: &str) -> PendingDelegationDigest {
    PendingDelegationDigest {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        updated_at: 0,
        rate: 0.0,
        principal: "".to_string(),
        cipher: "".to_string(),
        duration_minutes: 0,
        amount: 0.0,
        burst: 0,
        margin: 0.0,
        status: PendingDelegationDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_delegation_digest("code-140", "name-266");
        assert_eq!(item.status, PendingDelegationDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_pending_delegation_digest("code-140", "name-266");
        if item.move_to(PendingDelegationDigestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDelegationDigestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_delegation_digest("code-140", "name-266");
        item.status = PendingDelegationDigestStatus::Archived;
        assert!(
            item.move_to(PendingDelegationDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
