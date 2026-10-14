use super::*;

use acme_kit::text;

/// Where a pending migration slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingMigrationSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingMigrationSlotStatus {
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

impl std::fmt::Display for PendingMigrationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending migration slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationSlot {
    pub code: String,
    pub title: String,
    pub source: String,
    pub port: i64,
    pub tenant_id: String,
    pub quota_units: i64,
    pub status: PendingMigrationSlotStatus,
}

impl PendingMigrationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingMigrationSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingMigrationSlotStatus) -> bool {
        match self.status {
            PendingMigrationSlotStatus::Open => {
                match next {
                    PendingMigrationSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingMigrationSlotStatus::InReview => {
                match next {
                    PendingMigrationSlotStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingMigrationSlotStatus::Resolved => {
                return next == PendingMigrationSlotStatus::Reopened;
            }
            PendingMigrationSlotStatus::Reopened => {
                return next == PendingMigrationSlotStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingMigrationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending migration slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.quota_units as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingMigrationSlot {
        PendingMigrationSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            port: self.port,
            tenant_id: self.tenant_id.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending migration slot in status open.
pub fn new_pending_migration_slot(code: &str, title: &str) -> PendingMigrationSlot {
    PendingMigrationSlot {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        port: 0,
        tenant_id: "".to_string(),
        quota_units: 0,
        status: PendingMigrationSlotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_migration_slot("code-841", "title-208");
        assert_eq!(item.status, PendingMigrationSlotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_pending_migration_slot("code-841", "title-208");
        if item.move_to(PendingMigrationSlotStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingMigrationSlotStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_migration_slot("code-841", "title-208");
        item.status = PendingMigrationSlotStatus::Archived;
        assert!(
            item.move_to(PendingMigrationSlotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
