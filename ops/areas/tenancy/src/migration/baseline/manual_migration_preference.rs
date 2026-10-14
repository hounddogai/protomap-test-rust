use super::*;

use acme_kit::text;

/// Where a manual migration preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualMigrationPreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualMigrationPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualMigrationPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual migration preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationPreference {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub ceiling: f64,
    pub weight: f64,
    pub sequence: i64,
    pub notes: String,
    pub capacity: i64,
    pub status: ManualMigrationPreferenceStatus,
}

impl ManualMigrationPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualMigrationPreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualMigrationPreferenceStatus) -> bool {
        match self.status {
            ManualMigrationPreferenceStatus::Created => {
                match next {
                    ManualMigrationPreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualMigrationPreferenceStatus::Validated => {
                match next {
                    ManualMigrationPreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualMigrationPreferenceStatus::Published => {
                return next == ManualMigrationPreferenceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualMigrationPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual migration preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.capacity as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualMigrationPreference {
        ManualMigrationPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            ceiling: self.ceiling,
            weight: self.weight,
            sequence: self.sequence,
            notes: self.notes.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual migration preference in status created.
pub fn new_manual_migration_preference(code: &str, title: &str) -> ManualMigrationPreference {
    ManualMigrationPreference {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        ceiling: 0.0,
        weight: 0.0,
        sequence: 0,
        notes: "".to_string(),
        capacity: 0,
        status: ManualMigrationPreferenceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_migration_preference("code-681", "title-766");
        assert_eq!(item.status, ManualMigrationPreferenceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_manual_migration_preference("code-681", "title-766");
        if item.move_to(ManualMigrationPreferenceStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualMigrationPreferenceStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_migration_preference("code-681", "title-766");
        item.status = ManualMigrationPreferenceStatus::Retired;
        assert!(
            item.move_to(ManualMigrationPreferenceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
