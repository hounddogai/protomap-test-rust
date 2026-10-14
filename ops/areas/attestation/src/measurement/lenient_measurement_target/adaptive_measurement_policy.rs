use super::*;

use acme_kit::text;

/// Where a adaptive measurement policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveMeasurementPolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveMeasurementPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for AdaptiveMeasurementPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive measurement policy as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementPolicy {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub external_ref: String,
    pub issuer: String,
    pub principal: String,
    pub fingerprint: String,
    pub status: AdaptiveMeasurementPolicyStatus,
}

impl AdaptiveMeasurementPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveMeasurementPolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveMeasurementPolicyStatus) -> bool {
        match self.status {
            AdaptiveMeasurementPolicyStatus::Planned => {
                return next == AdaptiveMeasurementPolicyStatus::Completed;
            }
            AdaptiveMeasurementPolicyStatus::Scheduled => {
                match next {
                    AdaptiveMeasurementPolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveMeasurementPolicyStatus::InProgress => {
                return next == AdaptiveMeasurementPolicyStatus::Completed;
            }
            AdaptiveMeasurementPolicyStatus::Completed => {
                match next {
                    AdaptiveMeasurementPolicyStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: AdaptiveMeasurementPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive measurement policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive measurement policy", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> AdaptiveMeasurementPolicy {
        AdaptiveMeasurementPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            external_ref: self.external_ref.clone(),
            issuer: self.issuer.clone(),
            principal: self.principal.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive measurement policy in status planned.
pub fn new_adaptive_measurement_policy(code: &str, name: &str) -> AdaptiveMeasurementPolicy {
    AdaptiveMeasurementPolicy {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        external_ref: "".to_string(),
        issuer: "".to_string(),
        principal: "".to_string(),
        fingerprint: "".to_string(),
        status: AdaptiveMeasurementPolicyStatus::Planned,
    }
}
