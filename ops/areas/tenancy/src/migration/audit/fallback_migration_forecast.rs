use super::*;

use acme_kit::text;

/// Where a fallback migration forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackMigrationForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackMigrationForecastStatus {
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

impl std::fmt::Display for FallbackMigrationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback migration forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationForecast {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub title: String,
    pub discount: f64,
    pub notes: String,
    pub status: FallbackMigrationForecastStatus,
}

impl FallbackMigrationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackMigrationForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackMigrationForecastStatus) -> bool {
        match self.status {
            FallbackMigrationForecastStatus::Planned => {
                return next == FallbackMigrationForecastStatus::InProgress
                    || next == FallbackMigrationForecastStatus::Cancelled;
            }
            FallbackMigrationForecastStatus::Scheduled => {
                match next {
                    FallbackMigrationForecastStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackMigrationForecastStatus::InProgress => {
                return next == FallbackMigrationForecastStatus::Completed
                    || next == FallbackMigrationForecastStatus::Cancelled;
            }
            FallbackMigrationForecastStatus::Completed => {
                match next {
                    FallbackMigrationForecastStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FallbackMigrationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback migration forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> FallbackMigrationForecast {
        FallbackMigrationForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            title: self.title.clone(),
            discount: self.discount,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback migration forecast in status planned.
pub fn new_fallback_migration_forecast(code: &str, name: &str) -> FallbackMigrationForecast {
    FallbackMigrationForecast {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        title: "".to_string(),
        discount: 0.0,
        notes: "".to_string(),
        status: FallbackMigrationForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_migration_forecast("code-160", "name-650");
        assert_eq!(item.status, FallbackMigrationForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_fallback_migration_forecast("code-160", "name-650");
        if item.move_to(FallbackMigrationForecastStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackMigrationForecastStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_migration_forecast("code-160", "name-650");
        item.status = FallbackMigrationForecastStatus::Cancelled;
        assert!(
            item.move_to(FallbackMigrationForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
