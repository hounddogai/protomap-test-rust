use super::*;

use acme_kit::text;

/// Where a trust domain tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TrustDomainTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TrustDomainTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain tier as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTier {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub tenant_id: String,
    pub fingerprint: String,
    pub category: String,
    pub quantity: i64,
    pub status: TrustDomainTierStatus,
}

impl TrustDomainTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainTierStatus) -> bool {
        match self.status {
            TrustDomainTierStatus::Created => {
                return next == TrustDomainTierStatus::Validated;
            }
            TrustDomainTierStatus::Validated => {
                match next {
                    TrustDomainTierStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainTierStatus::Published => {
                match next {
                    TrustDomainTierStatus::Retired => {
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

    pub fn move_to(&mut self, next: TrustDomainTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "trust domain tier", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> TrustDomainTier {
        TrustDomainTier {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            category: self.category.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a trust domain tier in status created.
pub fn new_trust_domain_tier(code: &str, name: &str) -> TrustDomainTier {
    TrustDomainTier {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        category: "".to_string(),
        quantity: 0,
        status: TrustDomainTierStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_trust_domain_tier("code-435", "name-301");
        assert_eq!(item.status, TrustDomainTierStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_trust_domain_tier("code-435", "name-301");
        if item.move_to(TrustDomainTierStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainTierStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_trust_domain_tier("code-435", "name-301");
        item.status = TrustDomainTierStatus::Retired;
        assert!(
            item.move_to(TrustDomainTierStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
