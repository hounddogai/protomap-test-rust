use super::*;

use acme_kit::text;

/// Where a freeze entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FreezeEntryStatus {
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

impl std::fmt::Display for FreezeEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One freeze entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeEntry {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub maximum: f64,
    pub approved: bool,
    pub contact_email: String,
    pub created_at: i64,
    pub visible: bool,
    pub status: FreezeEntryStatus,
}

impl FreezeEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeEntryStatus) -> bool {
        match self.status {
            FreezeEntryStatus::New => {
                match next {
                    FreezeEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeEntryStatus::Queued => {
                return next == FreezeEntryStatus::Running || next == FreezeEntryStatus::Failed;
            }
            FreezeEntryStatus::Running => {
                return next == FreezeEntryStatus::Done || next == FreezeEntryStatus::Failed;
            }
            FreezeEntryStatus::Done => {
                return next == FreezeEntryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.created_at as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeEntry {
        FreezeEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            maximum: self.maximum,
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            created_at: self.created_at,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a freeze entry in status new.
pub fn new_freeze_entry(code: &str, label: &str) -> FreezeEntry {
    FreezeEntry {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        maximum: 0.0,
        approved: false,
        contact_email: "".to_string(),
        created_at: 0,
        visible: false,
        status: FreezeEntryStatus::New,
    }
}
