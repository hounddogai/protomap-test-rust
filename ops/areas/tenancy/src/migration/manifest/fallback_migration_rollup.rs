use super::*;

use acme_kit::text;

/// Where a fallback migration rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackMigrationRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackMigrationRollupStatus {
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

impl std::fmt::Display for FallbackMigrationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback migration rollup as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationRollup {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub budget: f64,
    pub category: String,
    pub ceiling: f64,
    pub issuer: String,
    pub region: String,
    pub status: FallbackMigrationRollupStatus,
}

impl FallbackMigrationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackMigrationRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackMigrationRollupStatus) -> bool {
        match self.status {
            FallbackMigrationRollupStatus::Planned => {
                match next {
                    FallbackMigrationRollupStatus::Completed | FallbackMigrationRollupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackMigrationRollupStatus::Scheduled => {
                return next == FallbackMigrationRollupStatus::InProgress
                    || next == FallbackMigrationRollupStatus::Cancelled;
            }
            FallbackMigrationRollupStatus::InProgress => {
                return next == FallbackMigrationRollupStatus::Cancelled;
            }
            FallbackMigrationRollupStatus::Completed => {
                return next == FallbackMigrationRollupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackMigrationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback migration rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackMigrationRollup {
        FallbackMigrationRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            budget: self.budget,
            category: self.category.clone(),
            ceiling: self.ceiling,
            issuer: self.issuer.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback migration rollup in status planned.
pub fn new_fallback_migration_rollup(code: &str, title: &str) -> FallbackMigrationRollup {
    FallbackMigrationRollup {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        budget: 0.0,
        category: "".to_string(),
        ceiling: 0.0,
        issuer: "".to_string(),
        region: "".to_string(),
        status: FallbackMigrationRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_migration_rollup("code-798", "title-151");
        assert_eq!(item.status, FallbackMigrationRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_fallback_migration_rollup("code-798", "title-151");
        if item.move_to(FallbackMigrationRollupStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackMigrationRollupStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_migration_rollup("code-798", "title-151");
        item.status = FallbackMigrationRollupStatus::Cancelled;
        assert!(
            item.move_to(FallbackMigrationRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
