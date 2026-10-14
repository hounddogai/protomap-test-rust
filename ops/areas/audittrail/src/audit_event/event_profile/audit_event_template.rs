use super::*;

use acme_kit::text;

/// Where a audit event template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AuditEventTemplateStatus {
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

impl std::fmt::Display for AuditEventTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event template as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTemplate {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub host: String,
    pub issuer: String,
    pub notes: String,
    pub revision: i64,
    pub status: AuditEventTemplateStatus,
}

impl AuditEventTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventTemplateStatus) -> bool {
        match self.status {
            AuditEventTemplateStatus::Open => {
                return next == AuditEventTemplateStatus::InReview || next == AuditEventTemplateStatus::Resolved;
            }
            AuditEventTemplateStatus::InReview => {
                return next == AuditEventTemplateStatus::Archived;
            }
            AuditEventTemplateStatus::Resolved => {
                match next {
                    AuditEventTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventTemplateStatus::Reopened => {
                match next {
                    AuditEventTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: AuditEventTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> AuditEventTemplate {
        AuditEventTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            host: self.host.clone(),
            issuer: self.issuer.clone(),
            notes: self.notes.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a audit event template in status open.
pub fn new_audit_event_template(code: &str, name: &str) -> AuditEventTemplate {
    AuditEventTemplate {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        host: "".to_string(),
        issuer: "".to_string(),
        notes: "".to_string(),
        revision: 0,
        status: AuditEventTemplateStatus::Open,
    }
}
