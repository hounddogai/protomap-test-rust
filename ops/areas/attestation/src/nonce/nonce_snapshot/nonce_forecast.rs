use super::*;

use acme_kit::text;

/// Where a nonce forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl NonceForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for NonceForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One nonce forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceForecast {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub expires_at: i64,
    pub external_ref: String,
    pub created_at: i64,
    pub contact_email: String,
    pub active: bool,
    pub archived: bool,
    pub status: NonceForecastStatus,
}

impl NonceForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceForecastStatus) -> bool {
        match self.status {
            NonceForecastStatus::Created => {
                return next == NonceForecastStatus::Validated || next == NonceForecastStatus::Published;
            }
            NonceForecastStatus::Validated => {
                return next == NonceForecastStatus::Published || next == NonceForecastStatus::Retired;
            }
            NonceForecastStatus::Published => {
                return next == NonceForecastStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce forecast {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.limit as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceForecast {
        NonceForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            created_at: self.created_at,
            contact_email: self.contact_email.clone(),
            active: self.active,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a nonce forecast in status created.
pub fn new_nonce_forecast(code: &str, label: &str) -> NonceForecast {
    NonceForecast {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        expires_at: 0,
        external_ref: "".to_string(),
        created_at: 0,
        contact_email: "".to_string(),
        active: false,
        archived: false,
        status: NonceForecastStatus::Created,
    }
}
