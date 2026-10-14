use super::*;

use acme_kit::text;

/// Where a capped policy audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPolicyAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedPolicyAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedPolicyAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped policy audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPolicyAudit {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub source: String,
    pub notes: String,
    pub archived: bool,
    pub status: CappedPolicyAuditStatus,
}

impl CappedPolicyAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPolicyAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPolicyAuditStatus) -> bool {
        match self.status {
            CappedPolicyAuditStatus::Requested => {
                return next == CappedPolicyAuditStatus::Expired;
            }
            CappedPolicyAuditStatus::Granted => {
                return next == CappedPolicyAuditStatus::Revoked;
            }
            CappedPolicyAuditStatus::Revoked => {
                match next {
                    CappedPolicyAuditStatus::Expired => {
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

    pub fn move_to(&mut self, next: CappedPolicyAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped policy audit {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> CappedPolicyAudit {
        CappedPolicyAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            source: self.source.clone(),
            notes: self.notes.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped policy audit in status requested.
pub fn new_capped_policy_audit(code: &str, label: &str) -> CappedPolicyAudit {
    CappedPolicyAudit {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        source: "".to_string(),
        notes: "".to_string(),
        archived: false,
        status: CappedPolicyAuditStatus::Requested,
    }
}
