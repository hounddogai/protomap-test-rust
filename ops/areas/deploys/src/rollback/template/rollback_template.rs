use super::*;

use acme_kit::text;

/// Where a rollback template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollbackTemplateStatus {
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

impl std::fmt::Display for RollbackTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTemplate {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub cipher: String,
    pub rotation_days: i64,
    pub channel: String,
    pub active: bool,
    pub ttl_seconds: i64,
    pub status: RollbackTemplateStatus,
}

impl RollbackTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackTemplateStatus) -> bool {
        match self.status {
            RollbackTemplateStatus::Open => {
                match next {
                    RollbackTemplateStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackTemplateStatus::InReview => {
                return next == RollbackTemplateStatus::Resolved || next == RollbackTemplateStatus::Archived;
            }
            RollbackTemplateStatus::Resolved => {
                match next {
                    RollbackTemplateStatus::Reopened | RollbackTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackTemplateStatus::Reopened => {
                match next {
                    RollbackTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollbackTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollback template", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.rotation_days as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackTemplate {
        RollbackTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            cipher: self.cipher.clone(),
            rotation_days: self.rotation_days,
            channel: self.channel.clone(),
            active: self.active,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rollback template in status open.
pub fn new_rollback_template(code: &str, name: &str) -> RollbackTemplate {
    RollbackTemplate {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        cipher: "".to_string(),
        rotation_days: 0,
        channel: "".to_string(),
        active: false,
        ttl_seconds: 0,
        status: RollbackTemplateStatus::Open,
    }
}
