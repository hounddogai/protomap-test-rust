use super::*;

use acme_kit::text;

/// Where a strict trust domain target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictTrustDomainTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictTrustDomainTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictTrustDomainTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict trust domain target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTrustDomainTarget {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub key_id: String,
    pub tenant_id: String,
    pub threshold: f64,
    pub notes: String,
    pub tags: Vec<String>,
    pub retries: i64,
    pub status: StrictTrustDomainTargetStatus,
}

impl StrictTrustDomainTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictTrustDomainTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictTrustDomainTargetStatus) -> bool {
        match self.status {
            StrictTrustDomainTargetStatus::Draft => {
                match next {
                    StrictTrustDomainTargetStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictTrustDomainTargetStatus::Pending => {
                return next == StrictTrustDomainTargetStatus::Suspended;
            }
            StrictTrustDomainTargetStatus::Active => {
                return next == StrictTrustDomainTargetStatus::Suspended
                    || next == StrictTrustDomainTargetStatus::Closed;
            }
            StrictTrustDomainTargetStatus::Suspended => {
                return next == StrictTrustDomainTargetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictTrustDomainTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict trust domain target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.threshold + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictTrustDomainTarget {
        StrictTrustDomainTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            key_id: self.key_id.clone(),
            tenant_id: self.tenant_id.clone(),
            threshold: self.threshold,
            notes: self.notes.clone(),
            tags: self.tags.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict trust domain target in status draft.
pub fn new_strict_trust_domain_target(code: &str, name: &str) -> StrictTrustDomainTarget {
    StrictTrustDomainTarget {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        key_id: "".to_string(),
        tenant_id: "".to_string(),
        threshold: 0.0,
        notes: "".to_string(),
        tags: Vec::new(),
        retries: 0,
        status: StrictTrustDomainTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_trust_domain_target("code-697", "name-690");
        assert_eq!(item.status, StrictTrustDomainTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_strict_trust_domain_target("code-697", "name-690");
        if item.move_to(StrictTrustDomainTargetStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictTrustDomainTargetStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_trust_domain_target("code-697", "name-690");
        item.status = StrictTrustDomainTargetStatus::Closed;
        assert!(
            item.move_to(StrictTrustDomainTargetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
