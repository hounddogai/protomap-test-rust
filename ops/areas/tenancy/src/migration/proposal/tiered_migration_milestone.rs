use super::*;

use acme_kit::text;

/// Where a tiered migration milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredMigrationMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredMigrationMilestoneStatus {
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

impl std::fmt::Display for TieredMigrationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered migration milestone as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMilestone {
    pub code: String,
    pub name: String,
    pub region: String,
    pub label: String,
    pub priority: i64,
    pub discount: f64,
    pub minimum: f64,
    pub score: f64,
    pub status: TieredMigrationMilestoneStatus,
}

impl TieredMigrationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredMigrationMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredMigrationMilestoneStatus) -> bool {
        match self.status {
            TieredMigrationMilestoneStatus::New => {
                match next {
                    TieredMigrationMilestoneStatus::Queued | TieredMigrationMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMigrationMilestoneStatus::Queued => {
                match next {
                    TieredMigrationMilestoneStatus::Running | TieredMigrationMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMigrationMilestoneStatus::Running => {
                return next == TieredMigrationMilestoneStatus::Done;
            }
            TieredMigrationMilestoneStatus::Done => {
                match next {
                    TieredMigrationMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredMigrationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered migration milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.priority as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredMigrationMilestone {
        TieredMigrationMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            label: self.label.clone(),
            priority: self.priority,
            discount: self.discount,
            minimum: self.minimum,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered migration milestone in status new.
pub fn new_tiered_migration_milestone(code: &str, name: &str) -> TieredMigrationMilestone {
    TieredMigrationMilestone {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        label: "".to_string(),
        priority: 0,
        discount: 0.0,
        minimum: 0.0,
        score: 0.0,
        status: TieredMigrationMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_migration_milestone("code-282", "name-731");
        assert_eq!(item.status, TieredMigrationMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tiered_migration_milestone("code-282", "name-731");
        if item.move_to(TieredMigrationMilestoneStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredMigrationMilestoneStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_migration_milestone("code-282", "name-731");
        item.status = TieredMigrationMilestoneStatus::Failed;
        assert!(
            item.move_to(TieredMigrationMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
