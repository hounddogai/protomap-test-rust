use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback trust domain estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackTrustDomainEstimateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackTrustDomainEstimateStatus {
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

impl std::fmt::Display for FallbackTrustDomainEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback trust domain estimate as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainEstimate {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub currency: String,
    pub limit: i64,
    pub rotation_days: i64,
    pub sequence: i64,
    pub quota_units: i64,
    pub retries: i64,
    pub issuer: String,
    pub status: FallbackTrustDomainEstimateStatus,
}

impl FallbackTrustDomainEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackTrustDomainEstimateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackTrustDomainEstimateStatus) -> bool {
        match self.status {
            FallbackTrustDomainEstimateStatus::Draft => {
                return next == FallbackTrustDomainEstimateStatus::Active;
            }
            FallbackTrustDomainEstimateStatus::Pending => {
                match next {
                    FallbackTrustDomainEstimateStatus::Active | FallbackTrustDomainEstimateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTrustDomainEstimateStatus::Active => {
                match next {
                    FallbackTrustDomainEstimateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTrustDomainEstimateStatus::Suspended => {
                match next {
                    FallbackTrustDomainEstimateStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackTrustDomainEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback trust domain estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.limit as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackTrustDomainEstimate {
        FallbackTrustDomainEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            currency: self.currency.clone(),
            limit: self.limit,
            rotation_days: self.rotation_days,
            sequence: self.sequence,
            quota_units: self.quota_units,
            retries: self.retries,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback trust domain estimate in status draft.
pub fn new_fallback_trust_domain_estimate(code: &str, label: &str) -> FallbackTrustDomainEstimate {
    FallbackTrustDomainEstimate {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        currency: "".to_string(),
        limit: 0,
        rotation_days: 0,
        sequence: 0,
        quota_units: 0,
        retries: 0,
        issuer: "".to_string(),
        status: FallbackTrustDomainEstimateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_trust_domain_estimate("code-50", "label-736");
        assert_eq!(item.status, FallbackTrustDomainEstimateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_fallback_trust_domain_estimate("code-50", "label-736");
        if item.move_to(FallbackTrustDomainEstimateStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackTrustDomainEstimateStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_trust_domain_estimate("code-50", "label-736");
        item.status = FallbackTrustDomainEstimateStatus::Closed;
        assert!(
            item.move_to(FallbackTrustDomainEstimateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
