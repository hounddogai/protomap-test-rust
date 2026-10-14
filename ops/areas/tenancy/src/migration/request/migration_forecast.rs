use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a migration forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl MigrationForecastStatus {
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

impl std::fmt::Display for MigrationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationForecast {
    pub code: String,
    pub name: String,
    pub source: String,
    pub attributes: BTreeMap<String, String>,
    pub grace_days: i64,
    pub status: MigrationForecastStatus,
}

impl MigrationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationForecastStatus) -> bool {
        match self.status {
            MigrationForecastStatus::Open => {
                match next {
                    MigrationForecastStatus::InReview | MigrationForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationForecastStatus::InReview => {
                return next == MigrationForecastStatus::Resolved || next == MigrationForecastStatus::Archived;
            }
            MigrationForecastStatus::Resolved => {
                return next == MigrationForecastStatus::Reopened || next == MigrationForecastStatus::Archived;
            }
            MigrationForecastStatus::Reopened => {
                match next {
                    MigrationForecastStatus::Archived => {
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

    pub fn move_to(&mut self, next: MigrationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> MigrationForecast {
        MigrationForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            attributes: self.attributes.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a migration forecast in status open.
pub fn new_migration_forecast(code: &str, name: &str) -> MigrationForecast {
    MigrationForecast {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        attributes: BTreeMap::new(),
        grace_days: 0,
        status: MigrationForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_migration_forecast("code-619", "name-737");
        assert_eq!(item.status, MigrationForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_migration_forecast("code-619", "name-737");
        if item.move_to(MigrationForecastStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationForecastStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_migration_forecast("code-619", "name-737");
        item.status = MigrationForecastStatus::Archived;
        assert!(
            item.move_to(MigrationForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
