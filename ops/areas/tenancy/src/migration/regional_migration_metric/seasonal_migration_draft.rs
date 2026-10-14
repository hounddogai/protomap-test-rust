use super::*;

use acme_kit::text;

/// Where a seasonal migration draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalMigrationDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalMigrationDraftStatus {
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

impl std::fmt::Display for SeasonalMigrationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal migration draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationDraft {
    pub code: String,
    pub name: String,
    pub region: String,
    pub ttl_seconds: i64,
    pub rate: f64,
    pub checksum: String,
    pub host: String,
    pub port: i64,
    pub scope: String,
    pub capacity: i64,
    pub status: SeasonalMigrationDraftStatus,
}

impl SeasonalMigrationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalMigrationDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalMigrationDraftStatus) -> bool {
        match self.status {
            SeasonalMigrationDraftStatus::Planned => {
                return next == SeasonalMigrationDraftStatus::Scheduled
                    || next == SeasonalMigrationDraftStatus::Completed;
            }
            SeasonalMigrationDraftStatus::Scheduled => {
                return next == SeasonalMigrationDraftStatus::Cancelled;
            }
            SeasonalMigrationDraftStatus::InProgress => {
                match next {
                    SeasonalMigrationDraftStatus::Completed | SeasonalMigrationDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalMigrationDraftStatus::Completed => {
                match next {
                    SeasonalMigrationDraftStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SeasonalMigrationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal migration draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal migration draft", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.rate - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalMigrationDraft {
        SeasonalMigrationDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            ttl_seconds: self.ttl_seconds,
            rate: self.rate,
            checksum: self.checksum.clone(),
            host: self.host.clone(),
            port: self.port,
            scope: self.scope.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a seasonal migration draft in status planned.
pub fn new_seasonal_migration_draft(code: &str, name: &str) -> SeasonalMigrationDraft {
    SeasonalMigrationDraft {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        ttl_seconds: 0,
        rate: 0.0,
        checksum: "".to_string(),
        host: "".to_string(),
        port: 0,
        scope: "".to_string(),
        capacity: 0,
        status: SeasonalMigrationDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_seasonal_migration_draft("code-81", "name-648");
        assert_eq!(item.status, SeasonalMigrationDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_seasonal_migration_draft("code-81", "name-648");
        if item.move_to(SeasonalMigrationDraftStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalMigrationDraftStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_seasonal_migration_draft("code-81", "name-648");
        item.status = SeasonalMigrationDraftStatus::Cancelled;
        assert!(
            item.move_to(SeasonalMigrationDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
