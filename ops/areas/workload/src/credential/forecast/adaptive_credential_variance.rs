use super::*;

use acme_kit::text;

/// Where a adaptive credential variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveCredentialVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveCredentialVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveCredentialVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive credential variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialVariance {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub audience: String,
    pub maximum: f64,
    pub rotation_days: i64,
    pub version: i64,
    pub pinned: bool,
    pub unit: String,
    pub retries: i64,
    pub status: AdaptiveCredentialVarianceStatus,
}

impl AdaptiveCredentialVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveCredentialVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveCredentialVarianceStatus) -> bool {
        match self.status {
            AdaptiveCredentialVarianceStatus::Proposed => {
                match next {
                    AdaptiveCredentialVarianceStatus::Approved | AdaptiveCredentialVarianceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveCredentialVarianceStatus::Approved => {
                return next == AdaptiveCredentialVarianceStatus::Rejected
                    || next == AdaptiveCredentialVarianceStatus::Withdrawn;
            }
            AdaptiveCredentialVarianceStatus::Rejected => {
                match next {
                    AdaptiveCredentialVarianceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveCredentialVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive credential variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive credential variance", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.maximum - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveCredentialVariance {
        AdaptiveCredentialVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            audience: self.audience.clone(),
            maximum: self.maximum,
            rotation_days: self.rotation_days,
            version: self.version,
            pinned: self.pinned,
            unit: self.unit.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a adaptive credential variance in status proposed.
pub fn new_adaptive_credential_variance(code: &str, title: &str) -> AdaptiveCredentialVariance {
    AdaptiveCredentialVariance {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        audience: "".to_string(),
        maximum: 0.0,
        rotation_days: 0,
        version: 0,
        pinned: false,
        unit: "".to_string(),
        retries: 0,
        status: AdaptiveCredentialVarianceStatus::Proposed,
    }
}
