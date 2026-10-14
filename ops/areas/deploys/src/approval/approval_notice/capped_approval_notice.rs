use super::*;

use acme_kit::text;

/// Where a capped approval notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedApprovalNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedApprovalNoticeStatus {
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

impl std::fmt::Display for CappedApprovalNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped approval notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedApprovalNotice {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub expires_at: i64,
    pub unit: String,
    pub audience: String,
    pub starts_at: i64,
    pub quota_units: i64,
    pub ttl_seconds: i64,
    pub priority: i64,
    pub status: CappedApprovalNoticeStatus,
}

impl CappedApprovalNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedApprovalNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedApprovalNoticeStatus) -> bool {
        match self.status {
            CappedApprovalNoticeStatus::Planned => {
                match next {
                    CappedApprovalNoticeStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedApprovalNoticeStatus::Scheduled => {
                match next {
                    CappedApprovalNoticeStatus::Completed | CappedApprovalNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedApprovalNoticeStatus::InProgress => {
                return next == CappedApprovalNoticeStatus::Cancelled;
            }
            CappedApprovalNoticeStatus::Completed => {
                return next == CappedApprovalNoticeStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedApprovalNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped approval notice {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.starts_at as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedApprovalNotice {
        CappedApprovalNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            expires_at: self.expires_at,
            unit: self.unit.clone(),
            audience: self.audience.clone(),
            starts_at: self.starts_at,
            quota_units: self.quota_units,
            ttl_seconds: self.ttl_seconds,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped approval notice in status planned.
pub fn new_capped_approval_notice(code: &str, title: &str) -> CappedApprovalNotice {
    CappedApprovalNotice {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        expires_at: 0,
        unit: "".to_string(),
        audience: "".to_string(),
        starts_at: 0,
        quota_units: 0,
        ttl_seconds: 0,
        priority: 0,
        status: CappedApprovalNoticeStatus::Planned,
    }
}
