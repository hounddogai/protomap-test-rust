use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a trust domain allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TrustDomainAllocationStatus {
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

impl std::fmt::Display for TrustDomainAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAllocation {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub owner_id: String,
    pub approved: bool,
    pub attributes: BTreeMap<String, String>,
    pub status: TrustDomainAllocationStatus,
}

impl TrustDomainAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainAllocationStatus) -> bool {
        match self.status {
            TrustDomainAllocationStatus::Open => {
                match next {
                    TrustDomainAllocationStatus::InReview | TrustDomainAllocationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainAllocationStatus::InReview => {
                match next {
                    TrustDomainAllocationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainAllocationStatus::Resolved => {
                match next {
                    TrustDomainAllocationStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainAllocationStatus::Reopened => {
                return next == TrustDomainAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> TrustDomainAllocation {
        TrustDomainAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            owner_id: self.owner_id.clone(),
            approved: self.approved,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a trust domain allocation in status open.
pub fn new_trust_domain_allocation(code: &str, name: &str) -> TrustDomainAllocation {
    TrustDomainAllocation {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        owner_id: "".to_string(),
        approved: false,
        attributes: BTreeMap::new(),
        status: TrustDomainAllocationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_trust_domain_allocation("code-223", "name-45");
        assert_eq!(item.status, TrustDomainAllocationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_trust_domain_allocation("code-223", "name-45");
        if item.move_to(TrustDomainAllocationStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainAllocationStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_trust_domain_allocation("code-223", "name-45");
        item.status = TrustDomainAllocationStatus::Archived;
        assert!(
            item.move_to(TrustDomainAllocationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
