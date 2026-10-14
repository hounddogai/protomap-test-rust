use super::*;

use acme_kit::text;

/// Where a migration rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MigrationRollupStatus {
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

impl std::fmt::Display for MigrationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration rollup as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRollup {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub country: String,
    pub pinned: bool,
    pub limit: i64,
    pub issuer: String,
    pub cluster: String,
    pub notes: String,
    pub status: MigrationRollupStatus,
}

impl MigrationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationRollupStatus) -> bool {
        match self.status {
            MigrationRollupStatus::New => {
                return next == MigrationRollupStatus::Queued || next == MigrationRollupStatus::Failed;
            }
            MigrationRollupStatus::Queued => {
                match next {
                    MigrationRollupStatus::Running | MigrationRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationRollupStatus::Running => {
                match next {
                    MigrationRollupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationRollupStatus::Done => {
                return next == MigrationRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> MigrationRollup {
        MigrationRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            country: self.country.clone(),
            pinned: self.pinned,
            limit: self.limit,
            issuer: self.issuer.clone(),
            cluster: self.cluster.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a migration rollup in status new.
pub fn new_migration_rollup(code: &str, title: &str) -> MigrationRollup {
    MigrationRollup {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        country: "".to_string(),
        pinned: false,
        limit: 0,
        issuer: "".to_string(),
        cluster: "".to_string(),
        notes: "".to_string(),
        status: MigrationRollupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_migration_rollup("code-178", "title-438");
        assert_eq!(item.status, MigrationRollupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_migration_rollup("code-178", "title-438");
        if item.move_to(MigrationRollupStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationRollupStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_migration_rollup("code-178", "title-438");
        item.status = MigrationRollupStatus::Failed;
        assert!(
            item.move_to(MigrationRollupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
