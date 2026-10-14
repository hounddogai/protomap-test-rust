use super::*;

use acme_kit::text;

/// Where a peering slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PeeringSlotStatus {
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

impl std::fmt::Display for PeeringSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a peering slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSlot {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub archived: bool,
    pub window_days: i64,
    pub sequence: i64,
    pub external_ref: String,
    pub title: String,
    pub scope: String,
    pub quantity: i64,
    pub ends_at: i64,
    pub status: PeeringSlotStatus,
}

impl PeeringSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringSlotStatus) -> bool {
        match self.status {
            PeeringSlotStatus::New => {
                match next {
                    PeeringSlotStatus::Queued | PeeringSlotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringSlotStatus::Queued => {
                return next == PeeringSlotStatus::Failed;
            }
            PeeringSlotStatus::Running => {
                return next == PeeringSlotStatus::Done || next == PeeringSlotStatus::Failed;
            }
            PeeringSlotStatus::Done => {
                return next == PeeringSlotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering slot {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.quantity as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringSlot {
        PeeringSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            archived: self.archived,
            window_days: self.window_days,
            sequence: self.sequence,
            external_ref: self.external_ref.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            quantity: self.quantity,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a peering slot in status new.
pub fn new_peering_slot(code: &str, label: &str) -> PeeringSlot {
    PeeringSlot {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        archived: false,
        window_days: 0,
        sequence: 0,
        external_ref: "".to_string(),
        title: "".to_string(),
        scope: "".to_string(),
        quantity: 0,
        ends_at: 0,
        status: PeeringSlotStatus::New,
    }
}
