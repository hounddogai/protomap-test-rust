use super::*;

use acme_kit::text;

/// Where a regional delegation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDelegationAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalDelegationAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RegionalDelegationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional delegation audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationAudit {
    pub code: String,
    pub name: String,
    pub label: String,
    pub created_at: i64,
    pub pinned: bool,
    pub host: String,
    pub duration_minutes: i64,
    pub ceiling: f64,
    pub archived: bool,
    pub channel: String,
    pub status: RegionalDelegationAuditStatus,
}

impl RegionalDelegationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDelegationAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDelegationAuditStatus) -> bool {
        match self.status {
            RegionalDelegationAuditStatus::Open => {
                match next {
                    RegionalDelegationAuditStatus::Reopened | RegionalDelegationAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDelegationAuditStatus::InReview => {
                match next {
                    RegionalDelegationAuditStatus::Resolved | RegionalDelegationAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDelegationAuditStatus::Resolved => {
                return next == RegionalDelegationAuditStatus::Reopened
                    || next == RegionalDelegationAuditStatus::Archived;
            }
            RegionalDelegationAuditStatus::Reopened => {
                return next == RegionalDelegationAuditStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalDelegationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional delegation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDelegationAudit {
        RegionalDelegationAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            pinned: self.pinned,
            host: self.host.clone(),
            duration_minutes: self.duration_minutes,
            ceiling: self.ceiling,
            archived: self.archived,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional delegation audit in status open.
pub fn new_regional_delegation_audit(code: &str, name: &str) -> RegionalDelegationAudit {
    RegionalDelegationAudit {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        created_at: 0,
        pinned: false,
        host: "".to_string(),
        duration_minutes: 0,
        ceiling: 0.0,
        archived: false,
        channel: "".to_string(),
        status: RegionalDelegationAuditStatus::Open,
    }
}
