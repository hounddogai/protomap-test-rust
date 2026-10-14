use super::*;

use acme_kit::text;

/// Where a trust domain variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TrustDomainVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TrustDomainVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain variance as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainVariance {
    pub code: String,
    pub title: String,
    pub region: String,
    pub created_at: i64,
    pub rotation_days: i64,
    pub issuer: String,
    pub starts_at: i64,
    pub tenant_id: String,
    pub status: TrustDomainVarianceStatus,
}

impl TrustDomainVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainVarianceStatus) -> bool {
        match self.status {
            TrustDomainVarianceStatus::Created => {
                match next {
                    TrustDomainVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainVarianceStatus::Validated => {
                return next == TrustDomainVarianceStatus::Retired;
            }
            TrustDomainVarianceStatus::Published => {
                match next {
                    TrustDomainVarianceStatus::Retired => {
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

    pub fn move_to(&mut self, next: TrustDomainVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "trust domain variance", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.created_at as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainVariance {
        TrustDomainVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            created_at: self.created_at,
            rotation_days: self.rotation_days,
            issuer: self.issuer.clone(),
            starts_at: self.starts_at,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a trust domain variance in status created.
pub fn new_trust_domain_variance(code: &str, title: &str) -> TrustDomainVariance {
    TrustDomainVariance {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        created_at: 0,
        rotation_days: 0,
        issuer: "".to_string(),
        starts_at: 0,
        tenant_id: "".to_string(),
        status: TrustDomainVarianceStatus::Created,
    }
}
