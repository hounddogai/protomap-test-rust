use super::*;

use acme_kit::text;

/// Where a forecast mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ForecastMappingStatus {
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

impl std::fmt::Display for ForecastMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast mapping as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMapping {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub grace_days: i64,
    pub description: String,
    pub window_days: i64,
    pub status: ForecastMappingStatus,
}

impl ForecastMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastMappingStatus) -> bool {
        match self.status {
            ForecastMappingStatus::Planned => {
                return next == ForecastMappingStatus::InProgress;
            }
            ForecastMappingStatus::Scheduled => {
                return next == ForecastMappingStatus::InProgress;
            }
            ForecastMappingStatus::InProgress => {
                return next == ForecastMappingStatus::Cancelled;
            }
            ForecastMappingStatus::Completed => {
                match next {
                    ForecastMappingStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ForecastMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "forecast mapping", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.window_days as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastMapping {
        ForecastMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            grace_days: self.grace_days,
            description: self.description.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a forecast mapping in status planned.
pub fn new_forecast_mapping(code: &str, title: &str) -> ForecastMapping {
    ForecastMapping {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        grace_days: 0,
        description: "".to_string(),
        window_days: 0,
        status: ForecastMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_forecast_mapping("code-928", "title-614");
        assert_eq!(item.status, ForecastMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_forecast_mapping("code-928", "title-614");
        if item.move_to(ForecastMappingStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastMappingStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_forecast_mapping("code-928", "title-614");
        item.status = ForecastMappingStatus::Cancelled;
        assert!(
            item.move_to(ForecastMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
