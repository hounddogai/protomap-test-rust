use super::*;

use acme_kit::text;

/// Where a scheduled migration segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledMigrationSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledMigrationSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledMigrationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled migration segment as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationSegment {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub summary: String,
    pub score: f64,
    pub status: ScheduledMigrationSegmentStatus,
}

impl ScheduledMigrationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledMigrationSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledMigrationSegmentStatus) -> bool {
        match self.status {
            ScheduledMigrationSegmentStatus::Proposed => {
                return next == ScheduledMigrationSegmentStatus::Rejected;
            }
            ScheduledMigrationSegmentStatus::Approved => {
                match next {
                    ScheduledMigrationSegmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledMigrationSegmentStatus::Rejected => {
                return next == ScheduledMigrationSegmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledMigrationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled migration segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.burst as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledMigrationSegment {
        ScheduledMigrationSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            summary: self.summary.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled migration segment in status proposed.
pub fn new_scheduled_migration_segment(code: &str, name: &str) -> ScheduledMigrationSegment {
    ScheduledMigrationSegment {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        summary: "".to_string(),
        score: 0.0,
        status: ScheduledMigrationSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_migration_segment("code-65", "name-302");
        assert_eq!(item.status, ScheduledMigrationSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_scheduled_migration_segment("code-65", "name-302");
        if item.move_to(ScheduledMigrationSegmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledMigrationSegmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_migration_segment("code-65", "name-302");
        item.status = ScheduledMigrationSegmentStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledMigrationSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
