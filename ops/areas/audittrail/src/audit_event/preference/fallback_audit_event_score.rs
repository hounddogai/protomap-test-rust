use super::*;

use acme_kit::text;

/// Where a fallback audit event score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackAuditEventScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackAuditEventScoreStatus {
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

impl std::fmt::Display for FallbackAuditEventScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback audit event score as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventScore {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub burst: i64,
    pub priority: i64,
    pub status: FallbackAuditEventScoreStatus,
}

impl FallbackAuditEventScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackAuditEventScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackAuditEventScoreStatus) -> bool {
        match self.status {
            FallbackAuditEventScoreStatus::Planned => {
                match next {
                    FallbackAuditEventScoreStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackAuditEventScoreStatus::Scheduled => {
                return next == FallbackAuditEventScoreStatus::Completed;
            }
            FallbackAuditEventScoreStatus::InProgress => {
                match next {
                    FallbackAuditEventScoreStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackAuditEventScoreStatus::Completed => {
                match next {
                    FallbackAuditEventScoreStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FallbackAuditEventScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback audit event score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackAuditEventScore {
        FallbackAuditEventScore {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            burst: self.burst,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback audit event score in status planned.
pub fn new_fallback_audit_event_score(code: &str, title: &str) -> FallbackAuditEventScore {
    FallbackAuditEventScore {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        burst: 0,
        priority: 0,
        status: FallbackAuditEventScoreStatus::Planned,
    }
}
