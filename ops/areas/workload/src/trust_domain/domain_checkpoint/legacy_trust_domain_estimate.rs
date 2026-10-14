use super::*;

use acme_kit::text;

/// Where a legacy trust domain estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTrustDomainEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyTrustDomainEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyTrustDomainEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy trust domain estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainEstimate {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub version: i64,
    pub rate: f64,
    pub priority: i64,
    pub unit: String,
    pub source: String,
    pub visible: bool,
    pub status: LegacyTrustDomainEstimateStatus,
}

impl LegacyTrustDomainEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTrustDomainEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTrustDomainEstimateStatus) -> bool {
        match self.status {
            LegacyTrustDomainEstimateStatus::Created => {
                match next {
                    LegacyTrustDomainEstimateStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTrustDomainEstimateStatus::Validated => {
                match next {
                    LegacyTrustDomainEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTrustDomainEstimateStatus::Published => {
                return next == LegacyTrustDomainEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyTrustDomainEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy trust domain estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.priority as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTrustDomainEstimate {
        LegacyTrustDomainEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            version: self.version,
            rate: self.rate,
            priority: self.priority,
            unit: self.unit.clone(),
            source: self.source.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy trust domain estimate in status created.
pub fn new_legacy_trust_domain_estimate(code: &str, name: &str) -> LegacyTrustDomainEstimate {
    LegacyTrustDomainEstimate {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        version: 0,
        rate: 0.0,
        priority: 0,
        unit: "".to_string(),
        source: "".to_string(),
        visible: false,
        status: LegacyTrustDomainEstimateStatus::Created,
    }
}
