use super::*;

use acme_kit::text;

/// Where a reservation policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ReservationPolicyStatus {
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

impl std::fmt::Display for ReservationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a reservation policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPolicy {
    pub code: String,
    pub name: String,
    pub category: String,
    pub archived: bool,
    pub active: bool,
    pub batch_size: i64,
    pub port: i64,
    pub priority: i64,
    pub pinned: bool,
    pub owner_id: String,
    pub status: ReservationPolicyStatus,
}

impl ReservationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationPolicyStatus) -> bool {
        match self.status {
            ReservationPolicyStatus::New => {
                match next {
                    ReservationPolicyStatus::Queued | ReservationPolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationPolicyStatus::Queued => {
                return next == ReservationPolicyStatus::Failed;
            }
            ReservationPolicyStatus::Running => {
                match next {
                    ReservationPolicyStatus::Done | ReservationPolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationPolicyStatus::Done => {
                return next == ReservationPolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.batch_size as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationPolicy {
        ReservationPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            archived: self.archived,
            active: self.active,
            batch_size: self.batch_size,
            port: self.port,
            priority: self.priority,
            pinned: self.pinned,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a reservation policy in status new.
pub fn new_reservation_policy(code: &str, name: &str) -> ReservationPolicy {
    ReservationPolicy {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        archived: false,
        active: false,
        batch_size: 0,
        port: 0,
        priority: 0,
        pinned: false,
        owner_id: "".to_string(),
        status: ReservationPolicyStatus::New,
    }
}
