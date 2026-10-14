use super::*;

use acme_kit::text;

/// Where a pending binding slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingBindingSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingBindingSlotStatus {
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

impl std::fmt::Display for PendingBindingSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending binding slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingSlot {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub floor_value: f64,
    pub source: String,
    pub ends_at: i64,
    pub currency: String,
    pub category: String,
    pub rotation_days: i64,
    pub starts_at: i64,
    pub score: f64,
    pub status: PendingBindingSlotStatus,
}

impl PendingBindingSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingBindingSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingBindingSlotStatus) -> bool {
        match self.status {
            PendingBindingSlotStatus::New => {
                match next {
                    PendingBindingSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingBindingSlotStatus::Queued => {
                return next == PendingBindingSlotStatus::Running || next == PendingBindingSlotStatus::Failed;
            }
            PendingBindingSlotStatus::Running => {
                match next {
                    PendingBindingSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingBindingSlotStatus::Done => {
                match next {
                    PendingBindingSlotStatus::Failed => {
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

    pub fn move_to(&mut self, next: PendingBindingSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending binding slot {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.starts_at as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingBindingSlot {
        PendingBindingSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            floor_value: self.floor_value,
            source: self.source.clone(),
            ends_at: self.ends_at,
            currency: self.currency.clone(),
            category: self.category.clone(),
            rotation_days: self.rotation_days,
            starts_at: self.starts_at,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending binding slot in status new.
pub fn new_pending_binding_slot(code: &str, name: &str) -> PendingBindingSlot {
    PendingBindingSlot {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        floor_value: 0.0,
        source: "".to_string(),
        ends_at: 0,
        currency: "".to_string(),
        category: "".to_string(),
        rotation_days: 0,
        starts_at: 0,
        score: 0.0,
        status: PendingBindingSlotStatus::New,
    }
}
