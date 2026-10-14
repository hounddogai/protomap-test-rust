use super::*;

use acme_kit::text;

/// Where a scheduled migration manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledMigrationManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledMigrationManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledMigrationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled migration manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationManifest {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub capacity: i64,
    pub reviewer: String,
    pub status: ScheduledMigrationManifestStatus,
}

impl ScheduledMigrationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledMigrationManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledMigrationManifestStatus) -> bool {
        match self.status {
            ScheduledMigrationManifestStatus::Created => {
                match next {
                    ScheduledMigrationManifestStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledMigrationManifestStatus::Validated => {
                return next == ScheduledMigrationManifestStatus::Published;
            }
            ScheduledMigrationManifestStatus::Published => {
                return next == ScheduledMigrationManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledMigrationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled migration manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> ScheduledMigrationManifest {
        ScheduledMigrationManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            capacity: self.capacity,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled migration manifest in status created.
pub fn new_scheduled_migration_manifest(code: &str, title: &str) -> ScheduledMigrationManifest {
    ScheduledMigrationManifest {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        capacity: 0,
        reviewer: "".to_string(),
        status: ScheduledMigrationManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_migration_manifest("code-68", "title-462");
        assert_eq!(item.status, ScheduledMigrationManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_scheduled_migration_manifest("code-68", "title-462");
        if item.move_to(ScheduledMigrationManifestStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledMigrationManifestStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_migration_manifest("code-68", "title-462");
        item.status = ScheduledMigrationManifestStatus::Retired;
        assert!(
            item.move_to(ScheduledMigrationManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
