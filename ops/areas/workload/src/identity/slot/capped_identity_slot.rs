use super::*;

use acme_kit::text;

/// Where a capped identity slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedIdentitySlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedIdentitySlotStatus {
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

impl std::fmt::Display for CappedIdentitySlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped identity slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedIdentitySlot {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub floor_value: f64,
    pub notes: String,
    pub category: String,
    pub threshold: f64,
    pub external_ref: String,
    pub checksum: String,
    pub summary: String,
    pub status: CappedIdentitySlotStatus,
}

impl CappedIdentitySlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedIdentitySlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedIdentitySlotStatus) -> bool {
        match self.status {
            CappedIdentitySlotStatus::Planned => {
                match next {
                    CappedIdentitySlotStatus::InProgress | CappedIdentitySlotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedIdentitySlotStatus::Scheduled => {
                return next == CappedIdentitySlotStatus::InProgress || next == CappedIdentitySlotStatus::Completed;
            }
            CappedIdentitySlotStatus::InProgress => {
                match next {
                    CappedIdentitySlotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedIdentitySlotStatus::Completed => {
                return next == CappedIdentitySlotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedIdentitySlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped identity slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped identity slot", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.floor_value + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedIdentitySlot {
        CappedIdentitySlot {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            floor_value: self.floor_value,
            notes: self.notes.clone(),
            category: self.category.clone(),
            threshold: self.threshold,
            external_ref: self.external_ref.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped identity slot in status planned.
pub fn new_capped_identity_slot(code: &str, title: &str) -> CappedIdentitySlot {
    CappedIdentitySlot {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        floor_value: 0.0,
        notes: "".to_string(),
        category: "".to_string(),
        threshold: 0.0,
        external_ref: "".to_string(),
        checksum: "".to_string(),
        summary: "".to_string(),
        status: CappedIdentitySlotStatus::Planned,
    }
}
