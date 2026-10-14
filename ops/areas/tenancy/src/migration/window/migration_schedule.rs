use super::*;

use acme_kit::text;

/// Where a migration schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MigrationScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MigrationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSchedule {
    pub code: String,
    pub label: String,
    pub title: String,
    pub quantity: i64,
    pub reviewer: String,
    pub status: MigrationScheduleStatus,
}

impl MigrationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationScheduleStatus) -> bool {
        match self.status {
            MigrationScheduleStatus::Proposed => {
                return next == MigrationScheduleStatus::Rejected || next == MigrationScheduleStatus::Withdrawn;
            }
            MigrationScheduleStatus::Approved => {
                match next {
                    MigrationScheduleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationScheduleStatus::Rejected => {
                match next {
                    MigrationScheduleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: MigrationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "migration schedule", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> MigrationSchedule {
        MigrationSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a migration schedule in status proposed.
pub fn new_migration_schedule(code: &str, label: &str) -> MigrationSchedule {
    MigrationSchedule {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        quantity: 0,
        reviewer: "".to_string(),
        status: MigrationScheduleStatus::Proposed,
    }
}
