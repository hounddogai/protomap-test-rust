use super::*;

use acme_kit::text;

/// Where a plan record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanRecordStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PlanRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PlanRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRecord {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub amount: f64,
    pub sequence: i64,
    pub pinned: bool,
    pub external_ref: String,
    pub duration_minutes: i64,
    pub status: PlanRecordStatus,
}

impl PlanRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanRecordStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanRecordStatus) -> bool {
        match self.status {
            PlanRecordStatus::Requested => {
                match next {
                    PlanRecordStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanRecordStatus::Granted => {
                match next {
                    PlanRecordStatus::Revoked | PlanRecordStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanRecordStatus::Revoked => {
                match next {
                    PlanRecordStatus::Expired => {
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

    pub fn move_to(&mut self, next: PlanRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.amount - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanRecord {
        PlanRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            amount: self.amount,
            sequence: self.sequence,
            pinned: self.pinned,
            external_ref: self.external_ref.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a plan record in status requested.
pub fn new_plan_record(code: &str, title: &str) -> PlanRecord {
    PlanRecord {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        amount: 0.0,
        sequence: 0,
        pinned: false,
        external_ref: "".to_string(),
        duration_minutes: 0,
        status: PlanRecordStatus::Requested,
    }
}
