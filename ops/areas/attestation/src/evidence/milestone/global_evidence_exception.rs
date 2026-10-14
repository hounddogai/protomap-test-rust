use super::*;

use acme_kit::text;

/// Where a global evidence exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalEvidenceExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalEvidenceExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GlobalEvidenceExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global evidence exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceException {
    pub code: String,
    pub title: String,
    pub label: String,
    pub contact_email: String,
    pub quantity: i64,
    pub status: GlobalEvidenceExceptionStatus,
}

impl GlobalEvidenceException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalEvidenceExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalEvidenceExceptionStatus) -> bool {
        match self.status {
            GlobalEvidenceExceptionStatus::Planned => {
                match next {
                    GlobalEvidenceExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEvidenceExceptionStatus::Scheduled => {
                match next {
                    GlobalEvidenceExceptionStatus::InProgress | GlobalEvidenceExceptionStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEvidenceExceptionStatus::InProgress => {
                return next == GlobalEvidenceExceptionStatus::Completed
                    || next == GlobalEvidenceExceptionStatus::Cancelled;
            }
            GlobalEvidenceExceptionStatus::Completed => {
                match next {
                    GlobalEvidenceExceptionStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalEvidenceExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global evidence exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> GlobalEvidenceException {
        GlobalEvidenceException {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global evidence exception in status planned.
pub fn new_global_evidence_exception(code: &str, title: &str) -> GlobalEvidenceException {
    GlobalEvidenceException {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        contact_email: "".to_string(),
        quantity: 0,
        status: GlobalEvidenceExceptionStatus::Planned,
    }
}
