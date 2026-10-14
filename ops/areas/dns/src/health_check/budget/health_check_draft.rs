use super::*;

use acme_kit::text;

/// Where a health check draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl HealthCheckDraftStatus {
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

impl std::fmt::Display for HealthCheckDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckDraft {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub ttl_seconds: i64,
    pub quantity: i64,
    pub status: HealthCheckDraftStatus,
}

impl HealthCheckDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckDraftStatus) -> bool {
        match self.status {
            HealthCheckDraftStatus::New => {
                return next == HealthCheckDraftStatus::Queued || next == HealthCheckDraftStatus::Running;
            }
            HealthCheckDraftStatus::Queued => {
                return next == HealthCheckDraftStatus::Running;
            }
            HealthCheckDraftStatus::Running => {
                return next == HealthCheckDraftStatus::Done || next == HealthCheckDraftStatus::Failed;
            }
            HealthCheckDraftStatus::Done => {
                return next == HealthCheckDraftStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.quantity as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckDraft {
        HealthCheckDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            ttl_seconds: self.ttl_seconds,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a health check draft in status new.
pub fn new_health_check_draft(code: &str, title: &str) -> HealthCheckDraft {
    HealthCheckDraft {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        ttl_seconds: 0,
        quantity: 0,
        status: HealthCheckDraftStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_health_check_draft("code-407", "title-173");
        assert_eq!(item.status, HealthCheckDraftStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_health_check_draft("code-407", "title-173");
        if item.move_to(HealthCheckDraftStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckDraftStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_health_check_draft("code-407", "title-173");
        item.status = HealthCheckDraftStatus::Failed;
        assert!(
            item.move_to(HealthCheckDraftStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
