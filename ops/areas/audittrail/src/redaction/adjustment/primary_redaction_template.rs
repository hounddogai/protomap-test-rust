use super::*;

use acme_kit::text;

/// Where a primary redaction template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRedactionTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryRedactionTemplateStatus {
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

impl std::fmt::Display for PrimaryRedactionTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary redaction template as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionTemplate {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub port: i64,
    pub issuer: String,
    pub status: PrimaryRedactionTemplateStatus,
}

impl PrimaryRedactionTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRedactionTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRedactionTemplateStatus) -> bool {
        match self.status {
            PrimaryRedactionTemplateStatus::Open => {
                return next == PrimaryRedactionTemplateStatus::Resolved
                    || next == PrimaryRedactionTemplateStatus::Archived;
            }
            PrimaryRedactionTemplateStatus::InReview => {
                return next == PrimaryRedactionTemplateStatus::Reopened;
            }
            PrimaryRedactionTemplateStatus::Resolved => {
                return next == PrimaryRedactionTemplateStatus::Reopened;
            }
            PrimaryRedactionTemplateStatus::Reopened => {
                match next {
                    PrimaryRedactionTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: PrimaryRedactionTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary redaction template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        (self.budget * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRedactionTemplate {
        PrimaryRedactionTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            port: self.port,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary redaction template in status open.
pub fn new_primary_redaction_template(code: &str, name: &str) -> PrimaryRedactionTemplate {
    PrimaryRedactionTemplate {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        port: 0,
        issuer: "".to_string(),
        status: PrimaryRedactionTemplateStatus::Open,
    }
}
