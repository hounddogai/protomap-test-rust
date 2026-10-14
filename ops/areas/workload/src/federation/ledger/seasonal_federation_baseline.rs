use super::*;

use acme_kit::text;

/// Where a seasonal federation baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFederationBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalFederationBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalFederationBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal federation baseline as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationBaseline {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub quantity: i64,
    pub maximum: f64,
    pub fingerprint: String,
    pub retries: i64,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub status: SeasonalFederationBaselineStatus,
}

impl SeasonalFederationBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFederationBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFederationBaselineStatus) -> bool {
        match self.status {
            SeasonalFederationBaselineStatus::Created => {
                return next == SeasonalFederationBaselineStatus::Published
                    || next == SeasonalFederationBaselineStatus::Retired;
            }
            SeasonalFederationBaselineStatus::Validated => {
                match next {
                    SeasonalFederationBaselineStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFederationBaselineStatus::Published => {
                match next {
                    SeasonalFederationBaselineStatus::Retired => {
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

    pub fn move_to(&mut self, next: SeasonalFederationBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal federation baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal federation baseline", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.maximum * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalFederationBaseline {
        SeasonalFederationBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            maximum: self.maximum,
            fingerprint: self.fingerprint.clone(),
            retries: self.retries,
            duration_minutes: self.duration_minutes,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal federation baseline in status created.
pub fn new_seasonal_federation_baseline(code: &str, label: &str) -> SeasonalFederationBaseline {
    SeasonalFederationBaseline {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        quantity: 0,
        maximum: 0.0,
        fingerprint: "".to_string(),
        retries: 0,
        duration_minutes: 0,
        capacity: 0,
        status: SeasonalFederationBaselineStatus::Created,
    }
}
