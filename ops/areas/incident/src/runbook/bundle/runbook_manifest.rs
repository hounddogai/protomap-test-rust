use super::*;

use acme_kit::text;

/// Where a runbook manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RunbookManifestStatus {
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

impl std::fmt::Display for RunbookManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook manifest as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookManifest {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub title: String,
    pub created_at: i64,
    pub source: String,
    pub amount: f64,
    pub status: RunbookManifestStatus,
}

impl RunbookManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookManifestStatus) -> bool {
        match self.status {
            RunbookManifestStatus::Open => {
                return next == RunbookManifestStatus::Resolved;
            }
            RunbookManifestStatus::InReview => {
                return next == RunbookManifestStatus::Resolved || next == RunbookManifestStatus::Reopened;
            }
            RunbookManifestStatus::Resolved => {
                match next {
                    RunbookManifestStatus::Reopened | RunbookManifestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookManifestStatus::Reopened => {
                match next {
                    RunbookManifestStatus::Archived => {
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

    pub fn move_to(&mut self, next: RunbookManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookManifest {
        RunbookManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            source: self.source.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a runbook manifest in status open.
pub fn new_runbook_manifest(code: &str, label: &str) -> RunbookManifest {
    RunbookManifest {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        title: "".to_string(),
        created_at: 0,
        source: "".to_string(),
        amount: 0.0,
        status: RunbookManifestStatus::Open,
    }
}
