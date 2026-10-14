use super::*;

use acme_kit::text;

/// Where a default reservation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReservationSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultReservationSlotStatus {
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

impl std::fmt::Display for DefaultReservationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default reservation slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSlot {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub batch_size: i64,
    pub category: String,
    pub notes: String,
    pub approved: bool,
    pub cipher: String,
    pub pinned: bool,
    pub cost: f64,
    pub cluster: String,
    pub status: DefaultReservationSlotStatus,
}

impl DefaultReservationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReservationSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReservationSlotStatus) -> bool {
        match self.status {
            DefaultReservationSlotStatus::Open => {
                match next {
                    DefaultReservationSlotStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReservationSlotStatus::InReview => {
                return next == DefaultReservationSlotStatus::Reopened;
            }
            DefaultReservationSlotStatus::Resolved => {
                return next == DefaultReservationSlotStatus::Reopened
                    || next == DefaultReservationSlotStatus::Archived;
            }
            DefaultReservationSlotStatus::Reopened => {
                match next {
                    DefaultReservationSlotStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultReservationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default reservation slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.batch_size as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultReservationSlot {
        DefaultReservationSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            batch_size: self.batch_size,
            category: self.category.clone(),
            notes: self.notes.clone(),
            approved: self.approved,
            cipher: self.cipher.clone(),
            pinned: self.pinned,
            cost: self.cost,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default reservation slot in status open.
pub fn new_default_reservation_slot(code: &str, name: &str) -> DefaultReservationSlot {
    DefaultReservationSlot {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        batch_size: 0,
        category: "".to_string(),
        notes: "".to_string(),
        approved: false,
        cipher: "".to_string(),
        pinned: false,
        cost: 0.0,
        cluster: "".to_string(),
        status: DefaultReservationSlotStatus::Open,
    }
}
