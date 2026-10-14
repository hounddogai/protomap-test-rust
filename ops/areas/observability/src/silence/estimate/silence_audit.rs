use super::*;

use acme_kit::text;

/// Where a silence audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SilenceAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SilenceAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a silence audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAudit {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub principal: String,
    pub approved: bool,
    pub threshold: f64,
    pub active: bool,
    pub budget: f64,
    pub expires_at: i64,
    pub status: SilenceAuditStatus,
}

impl SilenceAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceAuditStatus) -> bool {
        match self.status {
            SilenceAuditStatus::Proposed => {
                match next {
                    SilenceAuditStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceAuditStatus::Approved => {
                match next {
                    SilenceAuditStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceAuditStatus::Rejected => {
                return next == SilenceAuditStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence audit {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.budget * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceAudit {
        SilenceAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            principal: self.principal.clone(),
            approved: self.approved,
            threshold: self.threshold,
            active: self.active,
            budget: self.budget,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a silence audit in status proposed.
pub fn new_silence_audit(code: &str, name: &str) -> SilenceAudit {
    SilenceAudit {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        principal: "".to_string(),
        approved: false,
        threshold: 0.0,
        active: false,
        budget: 0.0,
        expires_at: 0,
        status: SilenceAuditStatus::Proposed,
    }
}
