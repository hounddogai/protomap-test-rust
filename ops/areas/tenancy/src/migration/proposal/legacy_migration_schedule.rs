use super::*;

use acme_kit::text;

/// Where a legacy migration schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyMigrationScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyMigrationScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LegacyMigrationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy migration schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationSchedule {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub threshold: f64,
    pub notes: String,
    pub description: String,
    pub floor_value: f64,
    pub source: String,
    pub locale: String,
    pub status: LegacyMigrationScheduleStatus,
}

impl LegacyMigrationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyMigrationScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyMigrationScheduleStatus) -> bool {
        match self.status {
            LegacyMigrationScheduleStatus::Open => {
                return next == LegacyMigrationScheduleStatus::Reopened;
            }
            LegacyMigrationScheduleStatus::InReview => {
                return next == LegacyMigrationScheduleStatus::Resolved
                    || next == LegacyMigrationScheduleStatus::Archived;
            }
            LegacyMigrationScheduleStatus::Resolved => {
                return next == LegacyMigrationScheduleStatus::Reopened
                    || next == LegacyMigrationScheduleStatus::Archived;
            }
            LegacyMigrationScheduleStatus::Reopened => {
                match next {
                    LegacyMigrationScheduleStatus::Archived => {
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

    pub fn move_to(&mut self, next: LegacyMigrationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy migration schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.threshold + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyMigrationSchedule {
        LegacyMigrationSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            threshold: self.threshold,
            notes: self.notes.clone(),
            description: self.description.clone(),
            floor_value: self.floor_value,
            source: self.source.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy migration schedule in status open.
pub fn new_legacy_migration_schedule(code: &str, label: &str) -> LegacyMigrationSchedule {
    LegacyMigrationSchedule {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        threshold: 0.0,
        notes: "".to_string(),
        description: "".to_string(),
        floor_value: 0.0,
        source: "".to_string(),
        locale: "".to_string(),
        status: LegacyMigrationScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_migration_schedule("code-307", "label-561");
        assert_eq!(item.status, LegacyMigrationScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_legacy_migration_schedule("code-307", "label-561");
        if item.move_to(LegacyMigrationScheduleStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyMigrationScheduleStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_migration_schedule("code-307", "label-561");
        item.status = LegacyMigrationScheduleStatus::Archived;
        assert!(
            item.move_to(LegacyMigrationScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
