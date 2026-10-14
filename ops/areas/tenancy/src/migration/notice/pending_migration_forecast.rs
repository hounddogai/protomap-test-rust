use super::*;

use acme_kit::text;

/// Where a pending migration forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingMigrationForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingMigrationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PendingMigrationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending migration forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationForecast {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub summary: String,
    pub margin: f64,
    pub audience: String,
    pub description: String,
    pub ends_at: i64,
    pub title: String,
    pub attempts: i64,
    pub name: String,
    pub status: PendingMigrationForecastStatus,
}

impl PendingMigrationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingMigrationForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingMigrationForecastStatus) -> bool {
        match self.status {
            PendingMigrationForecastStatus::Open => {
                match next {
                    PendingMigrationForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingMigrationForecastStatus::InReview => {
                match next {
                    PendingMigrationForecastStatus::Reopened | PendingMigrationForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingMigrationForecastStatus::Resolved => {
                match next {
                    PendingMigrationForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingMigrationForecastStatus::Reopened => {
                match next {
                    PendingMigrationForecastStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingMigrationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending migration forecast {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.attempts as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingMigrationForecast {
        PendingMigrationForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            summary: self.summary.clone(),
            margin: self.margin,
            audience: self.audience.clone(),
            description: self.description.clone(),
            ends_at: self.ends_at,
            title: self.title.clone(),
            attempts: self.attempts,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending migration forecast in status open.
pub fn new_pending_migration_forecast(code: &str, label: &str) -> PendingMigrationForecast {
    PendingMigrationForecast {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        summary: "".to_string(),
        margin: 0.0,
        audience: "".to_string(),
        description: "".to_string(),
        ends_at: 0,
        title: "".to_string(),
        attempts: 0,
        name: "".to_string(),
        status: PendingMigrationForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_migration_forecast("code-35", "label-296");
        assert_eq!(item.status, PendingMigrationForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_pending_migration_forecast("code-35", "label-296");
        if item.move_to(PendingMigrationForecastStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingMigrationForecastStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_migration_forecast("code-35", "label-296");
        item.status = PendingMigrationForecastStatus::Archived;
        assert!(
            item.move_to(PendingMigrationForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
