use super::*;

use acme_kit::text;

/// Where a capped trust domain assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTrustDomainAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedTrustDomainAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedTrustDomainAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped trust domain assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainAssignment {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub sequence: i64,
    pub capacity: i64,
    pub priority: i64,
    pub status: CappedTrustDomainAssignmentStatus,
}

impl CappedTrustDomainAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTrustDomainAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTrustDomainAssignmentStatus) -> bool {
        match self.status {
            CappedTrustDomainAssignmentStatus::Requested => {
                match next {
                    CappedTrustDomainAssignmentStatus::Revoked | CappedTrustDomainAssignmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTrustDomainAssignmentStatus::Granted => {
                match next {
                    CappedTrustDomainAssignmentStatus::Revoked | CappedTrustDomainAssignmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTrustDomainAssignmentStatus::Revoked => {
                return next == CappedTrustDomainAssignmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedTrustDomainAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped trust domain assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.priority as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTrustDomainAssignment {
        CappedTrustDomainAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            sequence: self.sequence,
            capacity: self.capacity,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped trust domain assignment in status requested.
pub fn new_capped_trust_domain_assignment(code: &str, title: &str) -> CappedTrustDomainAssignment {
    CappedTrustDomainAssignment {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        sequence: 0,
        capacity: 0,
        priority: 0,
        status: CappedTrustDomainAssignmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_trust_domain_assignment("code-328", "title-13");
        assert_eq!(item.status, CappedTrustDomainAssignmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_capped_trust_domain_assignment("code-328", "title-13");
        if item.move_to(CappedTrustDomainAssignmentStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTrustDomainAssignmentStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_trust_domain_assignment("code-328", "title-13");
        item.status = CappedTrustDomainAssignmentStatus::Expired;
        assert!(
            item.move_to(CappedTrustDomainAssignmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
