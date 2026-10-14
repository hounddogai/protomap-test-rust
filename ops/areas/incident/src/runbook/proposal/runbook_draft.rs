use super::*;

use acme_kit::text;

/// Where a runbook draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RunbookDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RunbookDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook draft as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDraft {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub retries: i64,
    pub archived: bool,
    pub rotation_days: i64,
    pub cipher: String,
    pub quantity: i64,
    pub burst: i64,
    pub country: String,
    pub status: RunbookDraftStatus,
}

impl RunbookDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookDraftStatus) -> bool {
        match self.status {
            RunbookDraftStatus::Draft => {
                return next == RunbookDraftStatus::Pending || next == RunbookDraftStatus::Closed;
            }
            RunbookDraftStatus::Pending => {
                match next {
                    RunbookDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookDraftStatus::Active => {
                return next == RunbookDraftStatus::Suspended || next == RunbookDraftStatus::Closed;
            }
            RunbookDraftStatus::Suspended => {
                match next {
                    RunbookDraftStatus::Closed => {
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

    pub fn move_to(&mut self, next: RunbookDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rotation_days as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookDraft {
        RunbookDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            retries: self.retries,
            archived: self.archived,
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            quantity: self.quantity,
            burst: self.burst,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a runbook draft in status draft.
pub fn new_runbook_draft(code: &str, label: &str) -> RunbookDraft {
    RunbookDraft {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        retries: 0,
        archived: false,
        rotation_days: 0,
        cipher: "".to_string(),
        quantity: 0,
        burst: 0,
        country: "".to_string(),
        status: RunbookDraftStatus::Draft,
    }
}
