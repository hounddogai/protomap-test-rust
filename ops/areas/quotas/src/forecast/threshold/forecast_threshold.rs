use super::*;

use acme_kit::text;

/// Where a forecast threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastThresholdStatus {
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

impl std::fmt::Display for ForecastThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast threshold as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastThreshold {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub ends_at: i64,
    pub maximum: f64,
    pub pinned: bool,
    pub expires_at: i64,
    pub grace_days: i64,
    pub active: bool,
    pub batch_size: i64,
    pub duration_minutes: i64,
    pub status: ForecastThresholdStatus,
}

impl ForecastThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastThresholdStatus) -> bool {
        match self.status {
            ForecastThresholdStatus::Draft => {
                return next == ForecastThresholdStatus::Active;
            }
            ForecastThresholdStatus::Pending => {
                return next == ForecastThresholdStatus::Active || next == ForecastThresholdStatus::Suspended;
            }
            ForecastThresholdStatus::Active => {
                return next == ForecastThresholdStatus::Suspended || next == ForecastThresholdStatus::Closed;
            }
            ForecastThresholdStatus::Suspended => {
                match next {
                    ForecastThresholdStatus::Closed => {
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

    pub fn move_to(&mut self, next: ForecastThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.expires_at as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastThreshold {
        ForecastThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            ends_at: self.ends_at,
            maximum: self.maximum,
            pinned: self.pinned,
            expires_at: self.expires_at,
            grace_days: self.grace_days,
            active: self.active,
            batch_size: self.batch_size,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a forecast threshold in status draft.
pub fn new_forecast_threshold(code: &str, name: &str) -> ForecastThreshold {
    ForecastThreshold {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        ends_at: 0,
        maximum: 0.0,
        pinned: false,
        expires_at: 0,
        grace_days: 0,
        active: false,
        batch_size: 0,
        duration_minutes: 0,
        status: ForecastThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_forecast_threshold("code-309", "name-481");
        assert_eq!(item.status, ForecastThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_forecast_threshold("code-309", "name-481");
        if item.move_to(ForecastThresholdStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastThresholdStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_forecast_threshold("code-309", "name-481");
        item.status = ForecastThresholdStatus::Closed;
        assert!(
            item.move_to(ForecastThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
