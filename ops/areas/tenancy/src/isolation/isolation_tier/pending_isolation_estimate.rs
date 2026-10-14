use super::*;

use acme_kit::text;

/// Where a pending isolation estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingIsolationEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingIsolationEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingIsolationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending isolation estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationEstimate {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub window_days: i64,
    pub ttl_seconds: i64,
    pub status: PendingIsolationEstimateStatus,
}

impl PendingIsolationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingIsolationEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingIsolationEstimateStatus) -> bool {
        match self.status {
            PendingIsolationEstimateStatus::Created => {
                return next == PendingIsolationEstimateStatus::Validated
                    || next == PendingIsolationEstimateStatus::Retired;
            }
            PendingIsolationEstimateStatus::Validated => {
                return next == PendingIsolationEstimateStatus::Published
                    || next == PendingIsolationEstimateStatus::Retired;
            }
            PendingIsolationEstimateStatus::Published => {
                match next {
                    PendingIsolationEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingIsolationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending isolation estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending isolation estimate", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingIsolationEstimate {
        PendingIsolationEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            window_days: self.window_days,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending isolation estimate in status created.
pub fn new_pending_isolation_estimate(code: &str, name: &str) -> PendingIsolationEstimate {
    PendingIsolationEstimate {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        window_days: 0,
        ttl_seconds: 0,
        status: PendingIsolationEstimateStatus::Created,
    }
}
