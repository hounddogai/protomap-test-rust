use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient trust domain variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTrustDomainVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientTrustDomainVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientTrustDomainVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient trust domain variance as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainVariance {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub notes: String,
    pub scope: String,
    pub attributes: BTreeMap<String, String>,
    pub budget: f64,
    pub status: LenientTrustDomainVarianceStatus,
}

impl LenientTrustDomainVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTrustDomainVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTrustDomainVarianceStatus) -> bool {
        match self.status {
            LenientTrustDomainVarianceStatus::Requested => {
                return next == LenientTrustDomainVarianceStatus::Expired;
            }
            LenientTrustDomainVarianceStatus::Granted => {
                return next == LenientTrustDomainVarianceStatus::Revoked;
            }
            LenientTrustDomainVarianceStatus::Revoked => {
                match next {
                    LenientTrustDomainVarianceStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientTrustDomainVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient trust domain variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn weighted_exposure(&self) -> f64 {
        ((self.rotation_days as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTrustDomainVariance {
        LenientTrustDomainVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            notes: self.notes.clone(),
            scope: self.scope.clone(),
            attributes: self.attributes.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient trust domain variance in status requested.
pub fn new_lenient_trust_domain_variance(code: &str, label: &str) -> LenientTrustDomainVariance {
    LenientTrustDomainVariance {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        notes: "".to_string(),
        scope: "".to_string(),
        attributes: BTreeMap::new(),
        budget: 0.0,
        status: LenientTrustDomainVarianceStatus::Requested,
    }
}
