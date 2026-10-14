use super::*;

use acme_kit::text;

/// Where a seasonal canary exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalCanaryExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalCanaryExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalCanaryExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal canary exception as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryException {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub external_ref: String,
    pub cluster: String,
    pub window_days: i64,
    pub grace_days: i64,
    pub burst: i64,
    pub notes: String,
    pub threshold: f64,
    pub status: SeasonalCanaryExceptionStatus,
}

impl SeasonalCanaryException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalCanaryExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalCanaryExceptionStatus) -> bool {
        match self.status {
            SeasonalCanaryExceptionStatus::Created => {
                return next == SeasonalCanaryExceptionStatus::Published;
            }
            SeasonalCanaryExceptionStatus::Validated => {
                match next {
                    SeasonalCanaryExceptionStatus::Published | SeasonalCanaryExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCanaryExceptionStatus::Published => {
                return next == SeasonalCanaryExceptionStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalCanaryExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal canary exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.window_days as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalCanaryException {
        SeasonalCanaryException {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            external_ref: self.external_ref.clone(),
            cluster: self.cluster.clone(),
            window_days: self.window_days,
            grace_days: self.grace_days,
            burst: self.burst,
            notes: self.notes.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal canary exception in status created.
pub fn new_seasonal_canary_exception(code: &str, label: &str) -> SeasonalCanaryException {
    SeasonalCanaryException {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        external_ref: "".to_string(),
        cluster: "".to_string(),
        window_days: 0,
        grace_days: 0,
        burst: 0,
        notes: "".to_string(),
        threshold: 0.0,
        status: SeasonalCanaryExceptionStatus::Created,
    }
}
