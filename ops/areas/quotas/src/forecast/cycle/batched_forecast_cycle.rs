use super::*;

use acme_kit::text;

/// Where a batched forecast cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedForecastCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedForecastCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedForecastCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched forecast cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastCycle {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub locale: String,
    pub cipher: String,
    pub audience: String,
    pub description: String,
    pub retries: i64,
    pub version: i64,
    pub status: BatchedForecastCycleStatus,
}

impl BatchedForecastCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedForecastCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedForecastCycleStatus) -> bool {
        match self.status {
            BatchedForecastCycleStatus::Created => {
                return next == BatchedForecastCycleStatus::Validated;
            }
            BatchedForecastCycleStatus::Validated => {
                return next == BatchedForecastCycleStatus::Retired;
            }
            BatchedForecastCycleStatus::Published => {
                return next == BatchedForecastCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedForecastCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched forecast cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.limit as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedForecastCycle {
        BatchedForecastCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            locale: self.locale.clone(),
            cipher: self.cipher.clone(),
            audience: self.audience.clone(),
            description: self.description.clone(),
            retries: self.retries,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched forecast cycle in status created.
pub fn new_batched_forecast_cycle(code: &str, label: &str) -> BatchedForecastCycle {
    BatchedForecastCycle {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        locale: "".to_string(),
        cipher: "".to_string(),
        audience: "".to_string(),
        description: "".to_string(),
        retries: 0,
        version: 0,
        status: BatchedForecastCycleStatus::Created,
    }
}
