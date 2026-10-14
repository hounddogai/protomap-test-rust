use super::*;

use acme_kit::text;

/// Where a credential audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CredentialAuditStatus {
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

impl std::fmt::Display for CredentialAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAudit {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub quota_units: i64,
    pub category: String,
    pub margin: f64,
    pub tags: Vec<String>,
    pub amount: f64,
    pub notes: String,
    pub checksum: String,
    pub status: CredentialAuditStatus,
}

impl CredentialAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialAuditStatus) -> bool {
        match self.status {
            CredentialAuditStatus::Open => {
                match next {
                    CredentialAuditStatus::InReview | CredentialAuditStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialAuditStatus::InReview => {
                return next == CredentialAuditStatus::Resolved || next == CredentialAuditStatus::Reopened;
            }
            CredentialAuditStatus::Resolved => {
                return next == CredentialAuditStatus::Reopened || next == CredentialAuditStatus::Archived;
            }
            CredentialAuditStatus::Reopened => {
                match next {
                    CredentialAuditStatus::Archived => {
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

    pub fn move_to(&mut self, next: CredentialAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        (self.margin * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialAudit {
        CredentialAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            quota_units: self.quota_units,
            category: self.category.clone(),
            margin: self.margin,
            tags: self.tags.clone(),
            amount: self.amount,
            notes: self.notes.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a credential audit in status open.
pub fn new_credential_audit(code: &str, title: &str) -> CredentialAudit {
    CredentialAudit {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        quota_units: 0,
        category: "".to_string(),
        margin: 0.0,
        tags: Vec::new(),
        amount: 0.0,
        notes: "".to_string(),
        checksum: "".to_string(),
        status: CredentialAuditStatus::Open,
    }
}
