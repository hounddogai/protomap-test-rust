use super::*;

use acme_kit::text;

/// Where a migration record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MigrationRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for MigrationRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRecord {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub grace_days: i64,
    pub unit: String,
    pub amount: f64,
    pub description: String,
    pub external_ref: String,
    pub namespace: String,
    pub status: MigrationRecordStatus,
}

impl MigrationRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationRecordStatus) -> bool {
        match self.status {
            MigrationRecordStatus::New => {
                match next {
                    MigrationRecordStatus::Queued | MigrationRecordStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationRecordStatus::Queued => {
                return next == MigrationRecordStatus::Done || next == MigrationRecordStatus::Failed;
            }
            MigrationRecordStatus::Running => {
                match next {
                    MigrationRecordStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationRecordStatus::Done => {
                return next == MigrationRecordStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "migration record", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.amount * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationRecord {
        MigrationRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            grace_days: self.grace_days,
            unit: self.unit.clone(),
            amount: self.amount,
            description: self.description.clone(),
            external_ref: self.external_ref.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a migration record in status new.
pub fn new_migration_record(code: &str, title: &str) -> MigrationRecord {
    MigrationRecord {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        grace_days: 0,
        unit: "".to_string(),
        amount: 0.0,
        description: "".to_string(),
        external_ref: "".to_string(),
        namespace: "".to_string(),
        status: MigrationRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_migration_record("code-257", "title-639");
        assert_eq!(item.status, MigrationRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_migration_record("code-257", "title-639");
        if item.move_to(MigrationRecordStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationRecordStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_migration_record("code-257", "title-639");
        item.status = MigrationRecordStatus::Failed;
        assert!(
            item.move_to(MigrationRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
