use super::*;

use acme_kit::text;

/// Where a rollback entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollbackEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RollbackEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEntry {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub ends_at: i64,
    pub visible: bool,
    pub status: RollbackEntryStatus,
}

impl RollbackEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackEntryStatus) -> bool {
        match self.status {
            RollbackEntryStatus::Draft => {
                return next == RollbackEntryStatus::Pending || next == RollbackEntryStatus::Suspended;
            }
            RollbackEntryStatus::Pending => {
                match next {
                    RollbackEntryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackEntryStatus::Active => {
                return next == RollbackEntryStatus::Suspended || next == RollbackEntryStatus::Closed;
            }
            RollbackEntryStatus::Suspended => {
                return next == RollbackEntryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.ends_at as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackEntry {
        RollbackEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            ends_at: self.ends_at,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollback entry in status draft.
pub fn new_rollback_entry(code: &str, name: &str) -> RollbackEntry {
    RollbackEntry {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        ends_at: 0,
        visible: false,
        status: RollbackEntryStatus::Draft,
    }
}
