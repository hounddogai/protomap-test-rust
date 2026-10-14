use super::*;

use acme_kit::text;

/// Where a peering record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringRecordStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PeeringRecordStatus {
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

impl std::fmt::Display for PeeringRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRecord {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub retries: i64,
    pub fingerprint: String,
    pub category: String,
    pub attempts: i64,
    pub quantity: i64,
    pub ratio: f64,
    pub status: PeeringRecordStatus,
}

impl PeeringRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringRecordStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringRecordStatus) -> bool {
        match self.status {
            PeeringRecordStatus::Planned => {
                match next {
                    PeeringRecordStatus::Scheduled | PeeringRecordStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringRecordStatus::Scheduled => {
                match next {
                    PeeringRecordStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringRecordStatus::InProgress => {
                match next {
                    PeeringRecordStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringRecordStatus::Completed => {
                match next {
                    PeeringRecordStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PeeringRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringRecord {
        PeeringRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            retries: self.retries,
            fingerprint: self.fingerprint.clone(),
            category: self.category.clone(),
            attempts: self.attempts,
            quantity: self.quantity,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a peering record in status planned.
pub fn new_peering_record(code: &str, label: &str) -> PeeringRecord {
    PeeringRecord {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        retries: 0,
        fingerprint: "".to_string(),
        category: "".to_string(),
        attempts: 0,
        quantity: 0,
        ratio: 0.0,
        status: PeeringRecordStatus::Planned,
    }
}
