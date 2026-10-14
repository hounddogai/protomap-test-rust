use super::*;

use acme_kit::text;

/// Where a failover record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FailoverRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FailoverRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRecord {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub unit: String,
    pub discount: f64,
    pub cipher: String,
    pub notes: String,
    pub channel: String,
    pub ends_at: i64,
    pub segment: String,
    pub cost: f64,
    pub status: FailoverRecordStatus,
}

impl FailoverRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverRecordStatus) -> bool {
        match self.status {
            FailoverRecordStatus::Proposed => {
                match next {
                    FailoverRecordStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverRecordStatus::Approved => {
                match next {
                    FailoverRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverRecordStatus::Rejected => {
                match next {
                    FailoverRecordStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FailoverRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover record {} cannot move from {} to {}",
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
        format!("{} {}", "failover record", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ends_at as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverRecord {
        FailoverRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            unit: self.unit.clone(),
            discount: self.discount,
            cipher: self.cipher.clone(),
            notes: self.notes.clone(),
            channel: self.channel.clone(),
            ends_at: self.ends_at,
            segment: self.segment.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a failover record in status proposed.
pub fn new_failover_record(code: &str, label: &str) -> FailoverRecord {
    FailoverRecord {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        unit: "".to_string(),
        discount: 0.0,
        cipher: "".to_string(),
        notes: "".to_string(),
        channel: "".to_string(),
        ends_at: 0,
        segment: "".to_string(),
        cost: 0.0,
        status: FailoverRecordStatus::Proposed,
    }
}
