use super::*;

use acme_kit::text;

/// Where a primary flow log record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFlowLogRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryFlowLogRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryFlowLogRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary flow log record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFlowLogRecord {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub amount: f64,
    pub expires_at: i64,
    pub key_id: String,
    pub status: PrimaryFlowLogRecordStatus,
}

impl PrimaryFlowLogRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFlowLogRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFlowLogRecordStatus) -> bool {
        match self.status {
            PrimaryFlowLogRecordStatus::Proposed => {
                match next {
                    PrimaryFlowLogRecordStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFlowLogRecordStatus::Approved => {
                match next {
                    PrimaryFlowLogRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFlowLogRecordStatus::Rejected => {
                return next == PrimaryFlowLogRecordStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryFlowLogRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary flow log record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.expires_at as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryFlowLogRecord {
        PrimaryFlowLogRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            amount: self.amount,
            expires_at: self.expires_at,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary flow log record in status proposed.
pub fn new_primary_flow_log_record(code: &str, label: &str) -> PrimaryFlowLogRecord {
    PrimaryFlowLogRecord {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        amount: 0.0,
        expires_at: 0,
        key_id: "".to_string(),
        status: PrimaryFlowLogRecordStatus::Proposed,
    }
}
