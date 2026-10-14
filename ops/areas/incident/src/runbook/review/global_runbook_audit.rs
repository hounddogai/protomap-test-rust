use super::*;

use acme_kit::text;

/// Where a global runbook audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRunbookAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalRunbookAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalRunbookAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global runbook audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRunbookAudit {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub limit: i64,
    pub active: bool,
    pub unit: String,
    pub description: String,
    pub priority: i64,
    pub duration_minutes: i64,
    pub status: GlobalRunbookAuditStatus,
}

impl GlobalRunbookAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRunbookAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRunbookAuditStatus) -> bool {
        match self.status {
            GlobalRunbookAuditStatus::Requested => {
                return next == GlobalRunbookAuditStatus::Granted || next == GlobalRunbookAuditStatus::Expired;
            }
            GlobalRunbookAuditStatus::Granted => {
                match next {
                    GlobalRunbookAuditStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRunbookAuditStatus::Revoked => {
                return next == GlobalRunbookAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalRunbookAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global runbook audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.duration_minutes as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRunbookAudit {
        GlobalRunbookAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            limit: self.limit,
            active: self.active,
            unit: self.unit.clone(),
            description: self.description.clone(),
            priority: self.priority,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a global runbook audit in status requested.
pub fn new_global_runbook_audit(code: &str, label: &str) -> GlobalRunbookAudit {
    GlobalRunbookAudit {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        limit: 0,
        active: false,
        unit: "".to_string(),
        description: "".to_string(),
        priority: 0,
        duration_minutes: 0,
        status: GlobalRunbookAuditStatus::Requested,
    }
}
