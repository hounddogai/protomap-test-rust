use super::*;

use acme_kit::text;

/// Where a capped redaction template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRedactionTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedRedactionTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedRedactionTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped redaction template as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionTemplate {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub cipher: String,
    pub notes: String,
    pub timezone: String,
    pub status: CappedRedactionTemplateStatus,
}

impl CappedRedactionTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRedactionTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRedactionTemplateStatus) -> bool {
        match self.status {
            CappedRedactionTemplateStatus::Created => {
                return next == CappedRedactionTemplateStatus::Validated
                    || next == CappedRedactionTemplateStatus::Published;
            }
            CappedRedactionTemplateStatus::Validated => {
                match next {
                    CappedRedactionTemplateStatus::Published | CappedRedactionTemplateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRedactionTemplateStatus::Published => {
                return next == CappedRedactionTemplateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRedactionTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped redaction template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> CappedRedactionTemplate {
        CappedRedactionTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            cipher: self.cipher.clone(),
            notes: self.notes.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped redaction template in status created.
pub fn new_capped_redaction_template(code: &str, name: &str) -> CappedRedactionTemplate {
    CappedRedactionTemplate {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        cipher: "".to_string(),
        notes: "".to_string(),
        timezone: "".to_string(),
        status: CappedRedactionTemplateStatus::Created,
    }
}
