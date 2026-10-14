use super::*;

use acme_kit::text;

/// Where a migration milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MigrationMilestoneStatus {
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

impl std::fmt::Display for MigrationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMilestone {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub ratio: f64,
    pub limit: i64,
    pub status: MigrationMilestoneStatus,
}

impl MigrationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationMilestoneStatus) -> bool {
        match self.status {
            MigrationMilestoneStatus::New => {
                return next == MigrationMilestoneStatus::Done || next == MigrationMilestoneStatus::Failed;
            }
            MigrationMilestoneStatus::Queued => {
                return next == MigrationMilestoneStatus::Failed;
            }
            MigrationMilestoneStatus::Running => {
                match next {
                    MigrationMilestoneStatus::Done | MigrationMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationMilestoneStatus::Done => {
                return next == MigrationMilestoneStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.limit as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationMilestone {
        MigrationMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            ratio: self.ratio,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a migration milestone in status new.
pub fn new_migration_milestone(code: &str, name: &str) -> MigrationMilestone {
    MigrationMilestone {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        ratio: 0.0,
        limit: 0,
        status: MigrationMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_migration_milestone("code-823", "name-726");
        assert_eq!(item.status, MigrationMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_migration_milestone("code-823", "name-726");
        if item.move_to(MigrationMilestoneStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationMilestoneStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_migration_milestone("code-823", "name-726");
        item.status = MigrationMilestoneStatus::Failed;
        assert!(
            item.move_to(MigrationMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
