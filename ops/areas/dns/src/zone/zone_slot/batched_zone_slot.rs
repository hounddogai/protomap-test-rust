use super::*;

use acme_kit::text;

/// Where a batched zone slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedZoneSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedZoneSlotStatus {
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

impl std::fmt::Display for BatchedZoneSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched zone slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneSlot {
    pub code: String,
    pub label: String,
    pub name: String,
    pub capacity: i64,
    pub cipher: String,
    pub principal: String,
    pub ttl_seconds: i64,
    pub status: BatchedZoneSlotStatus,
}

impl BatchedZoneSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedZoneSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedZoneSlotStatus) -> bool {
        match self.status {
            BatchedZoneSlotStatus::Planned => {
                return next == BatchedZoneSlotStatus::Completed;
            }
            BatchedZoneSlotStatus::Scheduled => {
                return next == BatchedZoneSlotStatus::InProgress || next == BatchedZoneSlotStatus::Cancelled;
            }
            BatchedZoneSlotStatus::InProgress => {
                return next == BatchedZoneSlotStatus::Completed || next == BatchedZoneSlotStatus::Cancelled;
            }
            BatchedZoneSlotStatus::Completed => {
                return next == BatchedZoneSlotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedZoneSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched zone slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.capacity as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedZoneSlot {
        BatchedZoneSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            cipher: self.cipher.clone(),
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched zone slot in status planned.
pub fn new_batched_zone_slot(code: &str, label: &str) -> BatchedZoneSlot {
    BatchedZoneSlot {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        capacity: 0,
        cipher: "".to_string(),
        principal: "".to_string(),
        ttl_seconds: 0,
        status: BatchedZoneSlotStatus::Planned,
    }
}
