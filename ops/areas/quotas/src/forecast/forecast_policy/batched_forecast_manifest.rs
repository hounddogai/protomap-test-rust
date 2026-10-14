use super::*;

use acme_kit::text;

/// Where a batched forecast manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedForecastManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedForecastManifestStatus {
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

impl std::fmt::Display for BatchedForecastManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched forecast manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastManifest {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub country: String,
    pub external_ref: String,
    pub maximum: f64,
    pub status: BatchedForecastManifestStatus,
}

impl BatchedForecastManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedForecastManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedForecastManifestStatus) -> bool {
        match self.status {
            BatchedForecastManifestStatus::Draft => {
                return next == BatchedForecastManifestStatus::Suspended;
            }
            BatchedForecastManifestStatus::Pending => {
                return next == BatchedForecastManifestStatus::Suspended;
            }
            BatchedForecastManifestStatus::Active => {
                return next == BatchedForecastManifestStatus::Closed;
            }
            BatchedForecastManifestStatus::Suspended => {
                match next {
                    BatchedForecastManifestStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedForecastManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched forecast manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.maximum - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedForecastManifest {
        BatchedForecastManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            country: self.country.clone(),
            external_ref: self.external_ref.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched forecast manifest in status draft.
pub fn new_batched_forecast_manifest(code: &str, title: &str) -> BatchedForecastManifest {
    BatchedForecastManifest {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        country: "".to_string(),
        external_ref: "".to_string(),
        maximum: 0.0,
        status: BatchedForecastManifestStatus::Draft,
    }
}
