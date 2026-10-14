use super::*;

use acme_kit::text;

/// Where a global forecast baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalForecastBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalForecastBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalForecastBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global forecast baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastBaseline {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub created_at: i64,
    pub revision: i64,
    pub archived: bool,
    pub status: GlobalForecastBaselineStatus,
}

impl GlobalForecastBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalForecastBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalForecastBaselineStatus) -> bool {
        match self.status {
            GlobalForecastBaselineStatus::Created => {
                match next {
                    GlobalForecastBaselineStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalForecastBaselineStatus::Validated => {
                return next == GlobalForecastBaselineStatus::Published;
            }
            GlobalForecastBaselineStatus::Published => {
                return next == GlobalForecastBaselineStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalForecastBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global forecast baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalForecastBaseline {
        GlobalForecastBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            created_at: self.created_at,
            revision: self.revision,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global forecast baseline in status created.
pub fn new_global_forecast_baseline(code: &str, title: &str) -> GlobalForecastBaseline {
    GlobalForecastBaseline {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        created_at: 0,
        revision: 0,
        archived: false,
        status: GlobalForecastBaselineStatus::Created,
    }
}
