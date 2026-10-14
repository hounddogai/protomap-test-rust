use super::*;

use acme_kit::text;

/// Where a global trust domain tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTrustDomainTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalTrustDomainTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalTrustDomainTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global trust domain tier as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainTier {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub contact_email: String,
    pub starts_at: i64,
    pub checksum: String,
    pub minimum: f64,
    pub status: GlobalTrustDomainTierStatus,
}

impl GlobalTrustDomainTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTrustDomainTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTrustDomainTierStatus) -> bool {
        match self.status {
            GlobalTrustDomainTierStatus::Proposed => {
                match next {
                    GlobalTrustDomainTierStatus::Approved | GlobalTrustDomainTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTrustDomainTierStatus::Approved => {
                return next == GlobalTrustDomainTierStatus::Rejected || next == GlobalTrustDomainTierStatus::Withdrawn;
            }
            GlobalTrustDomainTierStatus::Rejected => {
                return next == GlobalTrustDomainTierStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalTrustDomainTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global trust domain tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.minimum - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTrustDomainTier {
        GlobalTrustDomainTier {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            contact_email: self.contact_email.clone(),
            starts_at: self.starts_at,
            checksum: self.checksum.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global trust domain tier in status proposed.
pub fn new_global_trust_domain_tier(code: &str, name: &str) -> GlobalTrustDomainTier {
    GlobalTrustDomainTier {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        contact_email: "".to_string(),
        starts_at: 0,
        checksum: "".to_string(),
        minimum: 0.0,
        status: GlobalTrustDomainTierStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_global_trust_domain_tier("code-379", "name-32");
        assert_eq!(item.status, GlobalTrustDomainTierStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_global_trust_domain_tier("code-379", "name-32");
        if item.move_to(GlobalTrustDomainTierStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalTrustDomainTierStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_global_trust_domain_tier("code-379", "name-32");
        item.status = GlobalTrustDomainTierStatus::Withdrawn;
        assert!(
            item.move_to(GlobalTrustDomainTierStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
