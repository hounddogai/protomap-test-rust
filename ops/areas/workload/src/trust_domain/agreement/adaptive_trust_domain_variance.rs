use super::*;

use acme_kit::text;

/// Where a adaptive trust domain variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTrustDomainVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveTrustDomainVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveTrustDomainVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive trust domain variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainVariance {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub reviewer: String,
    pub locale: String,
    pub fingerprint: String,
    pub version: i64,
    pub status: AdaptiveTrustDomainVarianceStatus,
}

impl AdaptiveTrustDomainVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTrustDomainVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTrustDomainVarianceStatus) -> bool {
        match self.status {
            AdaptiveTrustDomainVarianceStatus::Proposed => {
                return next == AdaptiveTrustDomainVarianceStatus::Withdrawn;
            }
            AdaptiveTrustDomainVarianceStatus::Approved => {
                match next {
                    AdaptiveTrustDomainVarianceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveTrustDomainVarianceStatus::Rejected => {
                match next {
                    AdaptiveTrustDomainVarianceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveTrustDomainVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive trust domain variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> AdaptiveTrustDomainVariance {
        AdaptiveTrustDomainVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            reviewer: self.reviewer.clone(),
            locale: self.locale.clone(),
            fingerprint: self.fingerprint.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a adaptive trust domain variance in status proposed.
pub fn new_adaptive_trust_domain_variance(code: &str, name: &str) -> AdaptiveTrustDomainVariance {
    AdaptiveTrustDomainVariance {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        reviewer: "".to_string(),
        locale: "".to_string(),
        fingerprint: "".to_string(),
        version: 0,
        status: AdaptiveTrustDomainVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_trust_domain_variance("code-902", "name-653");
        assert_eq!(item.status, AdaptiveTrustDomainVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_adaptive_trust_domain_variance("code-902", "name-653");
        if item.move_to(AdaptiveTrustDomainVarianceStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveTrustDomainVarianceStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_trust_domain_variance("code-902", "name-653");
        item.status = AdaptiveTrustDomainVarianceStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveTrustDomainVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
