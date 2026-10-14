use super::*;

use acme_kit::text;

/// Where a tiered identity forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredIdentityForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredIdentityForecastStatus {
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

impl std::fmt::Display for TieredIdentityForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered identity forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityForecast {
    pub code: String,
    pub title: String,
    pub host: String,
    pub description: String,
    pub external_ref: String,
    pub expires_at: i64,
    pub status: TieredIdentityForecastStatus,
}

impl TieredIdentityForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredIdentityForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredIdentityForecastStatus) -> bool {
        match self.status {
            TieredIdentityForecastStatus::Planned => {
                return next == TieredIdentityForecastStatus::Completed;
            }
            TieredIdentityForecastStatus::Scheduled => {
                match next {
                    TieredIdentityForecastStatus::InProgress | TieredIdentityForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredIdentityForecastStatus::InProgress => {
                match next {
                    TieredIdentityForecastStatus::Completed | TieredIdentityForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredIdentityForecastStatus::Completed => {
                match next {
                    TieredIdentityForecastStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredIdentityForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered identity forecast {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> TieredIdentityForecast {
        TieredIdentityForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            description: self.description.clone(),
            external_ref: self.external_ref.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered identity forecast in status planned.
pub fn new_tiered_identity_forecast(code: &str, title: &str) -> TieredIdentityForecast {
    TieredIdentityForecast {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        description: "".to_string(),
        external_ref: "".to_string(),
        expires_at: 0,
        status: TieredIdentityForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_identity_forecast("code-404", "title-375");
        assert_eq!(item.status, TieredIdentityForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_tiered_identity_forecast("code-404", "title-375");
        if item.move_to(TieredIdentityForecastStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredIdentityForecastStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_identity_forecast("code-404", "title-375");
        item.status = TieredIdentityForecastStatus::Cancelled;
        assert!(
            item.move_to(TieredIdentityForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
