use super::*;

use acme_kit::text;

/// Where a redaction template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionTemplateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RedactionTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RedactionTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTemplate {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub locale: String,
    pub burst: i64,
    pub contact_email: String,
    pub status: RedactionTemplateStatus,
}

impl RedactionTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionTemplateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionTemplateStatus) -> bool {
        match self.status {
            RedactionTemplateStatus::New => {
                match next {
                    RedactionTemplateStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionTemplateStatus::Queued => {
                return next == RedactionTemplateStatus::Running;
            }
            RedactionTemplateStatus::Running => {
                match next {
                    RedactionTemplateStatus::Done | RedactionTemplateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionTemplateStatus::Done => {
                return next == RedactionTemplateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> RedactionTemplate {
        RedactionTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            locale: self.locale.clone(),
            burst: self.burst,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a redaction template in status new.
pub fn new_redaction_template(code: &str, label: &str) -> RedactionTemplate {
    RedactionTemplate {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        locale: "".to_string(),
        burst: 0,
        contact_email: "".to_string(),
        status: RedactionTemplateStatus::New,
    }
}
