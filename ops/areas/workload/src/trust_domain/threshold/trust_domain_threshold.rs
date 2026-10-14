use super::*;

use acme_kit::text;

/// Where a trust domain threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainThresholdStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TrustDomainThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TrustDomainThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain threshold as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainThreshold {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub region: String,
    pub starts_at: i64,
    pub status: TrustDomainThresholdStatus,
}

impl TrustDomainThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainThresholdStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainThresholdStatus) -> bool {
        match self.status {
            TrustDomainThresholdStatus::Created => {
                match next {
                    TrustDomainThresholdStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainThresholdStatus::Validated => {
                return next == TrustDomainThresholdStatus::Published || next == TrustDomainThresholdStatus::Retired;
            }
            TrustDomainThresholdStatus::Published => {
                return next == TrustDomainThresholdStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain threshold {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.starts_at as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainThreshold {
        TrustDomainThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            region: self.region.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a trust domain threshold in status created.
pub fn new_trust_domain_threshold(code: &str, title: &str) -> TrustDomainThreshold {
    TrustDomainThreshold {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        region: "".to_string(),
        starts_at: 0,
        status: TrustDomainThresholdStatus::Created,
    }
}
