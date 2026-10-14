use super::*;

use acme_kit::text;

/// Where a migration override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MigrationOverrideStatus {
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

impl std::fmt::Display for MigrationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One migration override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationOverride {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub pinned: bool,
    pub duration_minutes: i64,
    pub region: String,
    pub status: MigrationOverrideStatus,
}

impl MigrationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationOverrideStatus) -> bool {
        match self.status {
            MigrationOverrideStatus::New => {
                match next {
                    MigrationOverrideStatus::Done | MigrationOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationOverrideStatus::Queued => {
                match next {
                    MigrationOverrideStatus::Done | MigrationOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationOverrideStatus::Running => {
                return next == MigrationOverrideStatus::Done || next == MigrationOverrideStatus::Failed;
            }
            MigrationOverrideStatus::Done => {
                return next == MigrationOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> MigrationOverride {
        MigrationOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            pinned: self.pinned,
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a migration override in status new.
pub fn new_migration_override(code: &str, title: &str) -> MigrationOverride {
    MigrationOverride {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        pinned: false,
        duration_minutes: 0,
        region: "".to_string(),
        status: MigrationOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_migration_override("code-403", "title-879");
        assert_eq!(item.status, MigrationOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_migration_override("code-403", "title-879");
        if item.move_to(MigrationOverrideStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationOverrideStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_migration_override("code-403", "title-879");
        item.status = MigrationOverrideStatus::Failed;
        assert!(
            item.move_to(MigrationOverrideStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
