use super::*;

use acme_kit::text;

/// Where a global sink slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSinkSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalSinkSlotStatus {
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

impl std::fmt::Display for GlobalSinkSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global sink slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkSlot {
    pub code: String,
    pub name: String,
    pub country: String,
    pub floor_value: f64,
    pub score: f64,
    pub maximum: f64,
    pub status: GlobalSinkSlotStatus,
}

impl GlobalSinkSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSinkSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSinkSlotStatus) -> bool {
        match self.status {
            GlobalSinkSlotStatus::Open => {
                match next {
                    GlobalSinkSlotStatus::InReview | GlobalSinkSlotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSinkSlotStatus::InReview => {
                return next == GlobalSinkSlotStatus::Reopened || next == GlobalSinkSlotStatus::Archived;
            }
            GlobalSinkSlotStatus::Resolved => {
                match next {
                    GlobalSinkSlotStatus::Reopened | GlobalSinkSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSinkSlotStatus::Reopened => {
                return next == GlobalSinkSlotStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSinkSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global sink slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global sink slot", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.floor_value + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSinkSlot {
        GlobalSinkSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            floor_value: self.floor_value,
            score: self.score,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global sink slot in status open.
pub fn new_global_sink_slot(code: &str, name: &str) -> GlobalSinkSlot {
    GlobalSinkSlot {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        floor_value: 0.0,
        score: 0.0,
        maximum: 0.0,
        status: GlobalSinkSlotStatus::Open,
    }
}
