use super::*;

use acme_kit::text;

/// Where a primary trust domain allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTrustDomainAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryTrustDomainAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryTrustDomainAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary trust domain allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainAllocation {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub source: String,
    pub tenant_id: String,
    pub checksum: String,
    pub ends_at: i64,
    pub status: PrimaryTrustDomainAllocationStatus,
}

impl PrimaryTrustDomainAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTrustDomainAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTrustDomainAllocationStatus) -> bool {
        match self.status {
            PrimaryTrustDomainAllocationStatus::Requested => {
                match next {
                    PrimaryTrustDomainAllocationStatus::Revoked | PrimaryTrustDomainAllocationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTrustDomainAllocationStatus::Granted => {
                return next == PrimaryTrustDomainAllocationStatus::Revoked
                    || next == PrimaryTrustDomainAllocationStatus::Expired;
            }
            PrimaryTrustDomainAllocationStatus::Revoked => {
                return next == PrimaryTrustDomainAllocationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryTrustDomainAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary trust domain allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> PrimaryTrustDomainAllocation {
        PrimaryTrustDomainAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            source: self.source.clone(),
            tenant_id: self.tenant_id.clone(),
            checksum: self.checksum.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary trust domain allocation in status requested.
pub fn new_primary_trust_domain_allocation(code: &str, name: &str) -> PrimaryTrustDomainAllocation {
    PrimaryTrustDomainAllocation {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        source: "".to_string(),
        tenant_id: "".to_string(),
        checksum: "".to_string(),
        ends_at: 0,
        status: PrimaryTrustDomainAllocationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_trust_domain_allocation("code-21", "name-799");
        assert_eq!(item.status, PrimaryTrustDomainAllocationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_primary_trust_domain_allocation("code-21", "name-799");
        if item.move_to(PrimaryTrustDomainAllocationStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTrustDomainAllocationStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_trust_domain_allocation("code-21", "name-799");
        item.status = PrimaryTrustDomainAllocationStatus::Expired;
        assert!(
            item.move_to(PrimaryTrustDomainAllocationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
