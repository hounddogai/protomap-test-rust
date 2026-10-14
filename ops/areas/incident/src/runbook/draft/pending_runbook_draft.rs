use super::*;

use acme_kit::text;

/// Where a pending runbook draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRunbookDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingRunbookDraftStatus {
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

impl std::fmt::Display for PendingRunbookDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending runbook draft as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookDraft {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub port: i64,
    pub ratio: f64,
    pub scope: String,
    pub external_ref: String,
    pub host: String,
    pub timezone: String,
    pub ttl_seconds: i64,
    pub sequence: i64,
    pub status: PendingRunbookDraftStatus,
}

impl PendingRunbookDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRunbookDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRunbookDraftStatus) -> bool {
        match self.status {
            PendingRunbookDraftStatus::Planned => {
                match next {
                    PendingRunbookDraftStatus::Scheduled | PendingRunbookDraftStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRunbookDraftStatus::Scheduled => {
                return next == PendingRunbookDraftStatus::Completed;
            }
            PendingRunbookDraftStatus::InProgress => {
                match next {
                    PendingRunbookDraftStatus::Completed | PendingRunbookDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRunbookDraftStatus::Completed => {
                match next {
                    PendingRunbookDraftStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PendingRunbookDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending runbook draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending runbook draft", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRunbookDraft {
        PendingRunbookDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            port: self.port,
            ratio: self.ratio,
            scope: self.scope.clone(),
            external_ref: self.external_ref.clone(),
            host: self.host.clone(),
            timezone: self.timezone.clone(),
            ttl_seconds: self.ttl_seconds,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending runbook draft in status planned.
pub fn new_pending_runbook_draft(code: &str, name: &str) -> PendingRunbookDraft {
    PendingRunbookDraft {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        port: 0,
        ratio: 0.0,
        scope: "".to_string(),
        external_ref: "".to_string(),
        host: "".to_string(),
        timezone: "".to_string(),
        ttl_seconds: 0,
        sequence: 0,
        status: PendingRunbookDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_runbook_draft("code-254", "name-271");
        assert_eq!(item.status, PendingRunbookDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_pending_runbook_draft("code-254", "name-271");
        if item.move_to(PendingRunbookDraftStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRunbookDraftStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_runbook_draft("code-254", "name-271");
        item.status = PendingRunbookDraftStatus::Cancelled;
        assert!(
            item.move_to(PendingRunbookDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
