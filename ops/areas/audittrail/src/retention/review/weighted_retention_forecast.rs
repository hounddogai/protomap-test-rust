use super::*;

use acme_kit::text;

/// Where a weighted retention forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRetentionForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedRetentionForecastStatus {
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

impl std::fmt::Display for WeightedRetentionForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted retention forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionForecast {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub expires_at: i64,
    pub budget: f64,
    pub scope: String,
    pub pinned: bool,
    pub status: WeightedRetentionForecastStatus,
}

impl WeightedRetentionForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRetentionForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRetentionForecastStatus) -> bool {
        match self.status {
            WeightedRetentionForecastStatus::Draft => {
                return next == WeightedRetentionForecastStatus::Pending
                    || next == WeightedRetentionForecastStatus::Active;
            }
            WeightedRetentionForecastStatus::Pending => {
                return next == WeightedRetentionForecastStatus::Active;
            }
            WeightedRetentionForecastStatus::Active => {
                match next {
                    WeightedRetentionForecastStatus::Suspended | WeightedRetentionForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRetentionForecastStatus::Suspended => {
                return next == WeightedRetentionForecastStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRetentionForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted retention forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        (self.budget * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRetentionForecast {
        WeightedRetentionForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            expires_at: self.expires_at,
            budget: self.budget,
            scope: self.scope.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted retention forecast in status draft.
pub fn new_weighted_retention_forecast(code: &str, label: &str) -> WeightedRetentionForecast {
    WeightedRetentionForecast {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        expires_at: 0,
        budget: 0.0,
        scope: "".to_string(),
        pinned: false,
        status: WeightedRetentionForecastStatus::Draft,
    }
}
