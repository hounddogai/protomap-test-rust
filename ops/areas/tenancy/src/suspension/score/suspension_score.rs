use super::*;

use acme_kit::text;

/// Where a suspension score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SuspensionScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SuspensionScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScore {
    pub code: String,
    pub label: String,
    pub description: String,
    pub discount: f64,
    pub threshold: f64,
    pub contact_email: String,
    pub scope: String,
    pub duration_minutes: i64,
    pub status: SuspensionScoreStatus,
}

impl SuspensionScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionScoreStatus) -> bool {
        match self.status {
            SuspensionScoreStatus::Created => {
                return next == SuspensionScoreStatus::Validated || next == SuspensionScoreStatus::Retired;
            }
            SuspensionScoreStatus::Validated => {
                return next == SuspensionScoreStatus::Retired;
            }
            SuspensionScoreStatus::Published => {
                return next == SuspensionScoreStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "suspension score", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.discount * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionScore {
        SuspensionScore {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            discount: self.discount,
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            scope: self.scope.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a suspension score in status created.
pub fn new_suspension_score(code: &str, label: &str) -> SuspensionScore {
    SuspensionScore {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        discount: 0.0,
        threshold: 0.0,
        contact_email: "".to_string(),
        scope: "".to_string(),
        duration_minutes: 0,
        status: SuspensionScoreStatus::Created,
    }
}
