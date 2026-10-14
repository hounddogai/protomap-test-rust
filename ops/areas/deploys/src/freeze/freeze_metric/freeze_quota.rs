use super::*;

use acme_kit::text;

/// Where a freeze quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FreezeQuotaStatus {
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

impl std::fmt::Display for FreezeQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One freeze quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeQuota {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub quantity: i64,
    pub notes: String,
    pub reviewer: String,
    pub key_id: String,
    pub capacity: i64,
    pub status: FreezeQuotaStatus,
}

impl FreezeQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeQuotaStatus) -> bool {
        match self.status {
            FreezeQuotaStatus::Planned => {
                return next == FreezeQuotaStatus::Completed || next == FreezeQuotaStatus::Cancelled;
            }
            FreezeQuotaStatus::Scheduled => {
                match next {
                    FreezeQuotaStatus::InProgress | FreezeQuotaStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeQuotaStatus::InProgress => {
                match next {
                    FreezeQuotaStatus::Completed | FreezeQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeQuotaStatus::Completed => {
                return next == FreezeQuotaStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quantity as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeQuota {
        FreezeQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            quantity: self.quantity,
            notes: self.notes.clone(),
            reviewer: self.reviewer.clone(),
            key_id: self.key_id.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a freeze quota in status planned.
pub fn new_freeze_quota(code: &str, title: &str) -> FreezeQuota {
    FreezeQuota {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        quantity: 0,
        notes: "".to_string(),
        reviewer: "".to_string(),
        key_id: "".to_string(),
        capacity: 0,
        status: FreezeQuotaStatus::Planned,
    }
}
