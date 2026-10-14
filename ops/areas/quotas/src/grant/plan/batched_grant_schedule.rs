use super::*;

use acme_kit::text;

/// Where a batched grant schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedGrantScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedGrantScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedGrantScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched grant schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantSchedule {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub title: String,
    pub contact_email: String,
    pub revision: i64,
    pub locale: String,
    pub status: BatchedGrantScheduleStatus,
}

impl BatchedGrantSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedGrantScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedGrantScheduleStatus) -> bool {
        match self.status {
            BatchedGrantScheduleStatus::Requested => {
                return next == BatchedGrantScheduleStatus::Granted || next == BatchedGrantScheduleStatus::Expired;
            }
            BatchedGrantScheduleStatus::Granted => {
                match next {
                    BatchedGrantScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedGrantScheduleStatus::Revoked => {
                match next {
                    BatchedGrantScheduleStatus::Expired => {
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

    pub fn move_to(&mut self, next: BatchedGrantScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched grant schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.margin - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedGrantSchedule {
        BatchedGrantSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            revision: self.revision,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched grant schedule in status requested.
pub fn new_batched_grant_schedule(code: &str, name: &str) -> BatchedGrantSchedule {
    BatchedGrantSchedule {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        title: "".to_string(),
        contact_email: "".to_string(),
        revision: 0,
        locale: "".to_string(),
        status: BatchedGrantScheduleStatus::Requested,
    }
}
