use super::*;

use acme_kit::text;

/// Where a rolling isolation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingIsolationAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingIsolationAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingIsolationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling isolation audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationAudit {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub retries: i64,
    pub sequence: i64,
    pub status: RollingIsolationAuditStatus,
}

impl RollingIsolationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingIsolationAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingIsolationAuditStatus) -> bool {
        match self.status {
            RollingIsolationAuditStatus::Proposed => {
                match next {
                    RollingIsolationAuditStatus::Approved | RollingIsolationAuditStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingIsolationAuditStatus::Approved => {
                match next {
                    RollingIsolationAuditStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingIsolationAuditStatus::Rejected => {
                match next {
                    RollingIsolationAuditStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingIsolationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling isolation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.retries as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingIsolationAudit {
        RollingIsolationAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            retries: self.retries,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling isolation audit in status proposed.
pub fn new_rolling_isolation_audit(code: &str, label: &str) -> RollingIsolationAudit {
    RollingIsolationAudit {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        retries: 0,
        sequence: 0,
        status: RollingIsolationAuditStatus::Proposed,
    }
}
