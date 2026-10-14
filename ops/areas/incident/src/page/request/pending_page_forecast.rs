use super::*;

use acme_kit::text;

/// Where a pending page forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPageForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingPageForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingPageForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending page forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageForecast {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub ratio: f64,
    pub maximum: f64,
    pub segment: String,
    pub summary: String,
    pub archived: bool,
    pub phone: String,
    pub host: String,
    pub created_at: i64,
    pub status: PendingPageForecastStatus,
}

impl PendingPageForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPageForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPageForecastStatus) -> bool {
        match self.status {
            PendingPageForecastStatus::Created => {
                match next {
                    PendingPageForecastStatus::Published | PendingPageForecastStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPageForecastStatus::Validated => {
                match next {
                    PendingPageForecastStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPageForecastStatus::Published => {
                return next == PendingPageForecastStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingPageForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending page forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending page forecast", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ratio - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPageForecast {
        PendingPageForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            ratio: self.ratio,
            maximum: self.maximum,
            segment: self.segment.clone(),
            summary: self.summary.clone(),
            archived: self.archived,
            phone: self.phone.clone(),
            host: self.host.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a pending page forecast in status created.
pub fn new_pending_page_forecast(code: &str, label: &str) -> PendingPageForecast {
    PendingPageForecast {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        ratio: 0.0,
        maximum: 0.0,
        segment: "".to_string(),
        summary: "".to_string(),
        archived: false,
        phone: "".to_string(),
        host: "".to_string(),
        created_at: 0,
        status: PendingPageForecastStatus::Created,
    }
}
