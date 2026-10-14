use super::*;

use acme_kit::text;

/// Where a runbook request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookRequestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RunbookRequestStatus {
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

impl std::fmt::Display for RunbookRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookRequest {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub tags: Vec<String>,
    pub tenant_id: String,
    pub score: f64,
    pub status: RunbookRequestStatus,
}

impl RunbookRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookRequestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookRequestStatus) -> bool {
        match self.status {
            RunbookRequestStatus::Planned => {
                return next == RunbookRequestStatus::Completed;
            }
            RunbookRequestStatus::Scheduled => {
                return next == RunbookRequestStatus::InProgress || next == RunbookRequestStatus::Completed;
            }
            RunbookRequestStatus::InProgress => {
                return next == RunbookRequestStatus::Completed || next == RunbookRequestStatus::Cancelled;
            }
            RunbookRequestStatus::Completed => {
                match next {
                    RunbookRequestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RunbookRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.rotation_days as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookRequest {
        RunbookRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            tags: self.tags.clone(),
            tenant_id: self.tenant_id.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a runbook request in status planned.
pub fn new_runbook_request(code: &str, title: &str) -> RunbookRequest {
    RunbookRequest {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        tags: Vec::new(),
        tenant_id: "".to_string(),
        score: 0.0,
        status: RunbookRequestStatus::Planned,
    }
}
