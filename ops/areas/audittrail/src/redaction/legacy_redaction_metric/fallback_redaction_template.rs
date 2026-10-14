use super::*;

use acme_kit::text;

/// Where a fallback redaction template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRedactionTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackRedactionTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackRedactionTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback redaction template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionTemplate {
    pub code: String,
    pub title: String,
    pub category: String,
    pub unit: String,
    pub approved: bool,
    pub limit: i64,
    pub discount: f64,
    pub status: FallbackRedactionTemplateStatus,
}

impl FallbackRedactionTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRedactionTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRedactionTemplateStatus) -> bool {
        match self.status {
            FallbackRedactionTemplateStatus::Proposed => {
                return next == FallbackRedactionTemplateStatus::Rejected
                    || next == FallbackRedactionTemplateStatus::Withdrawn;
            }
            FallbackRedactionTemplateStatus::Approved => {
                return next == FallbackRedactionTemplateStatus::Rejected
                    || next == FallbackRedactionTemplateStatus::Withdrawn;
            }
            FallbackRedactionTemplateStatus::Rejected => {
                match next {
                    FallbackRedactionTemplateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackRedactionTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback redaction template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback redaction template", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.discount * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRedactionTemplate {
        FallbackRedactionTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            unit: self.unit.clone(),
            approved: self.approved,
            limit: self.limit,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback redaction template in status proposed.
pub fn new_fallback_redaction_template(code: &str, title: &str) -> FallbackRedactionTemplate {
    FallbackRedactionTemplate {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        unit: "".to_string(),
        approved: false,
        limit: 0,
        discount: 0.0,
        status: FallbackRedactionTemplateStatus::Proposed,
    }
}
