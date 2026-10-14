use super::*;

use acme_kit::text;

/// Where a runbook override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RunbookOverrideStatus {
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

impl std::fmt::Display for RunbookOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookOverride {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub tenant_id: String,
    pub phone: String,
    pub priority: i64,
    pub scope: String,
    pub country: String,
    pub updated_at: i64,
    pub score: f64,
    pub pinned: bool,
    pub status: RunbookOverrideStatus,
}

impl RunbookOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookOverrideStatus) -> bool {
        match self.status {
            RunbookOverrideStatus::New => {
                match next {
                    RunbookOverrideStatus::Running | RunbookOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookOverrideStatus::Queued => {
                return next == RunbookOverrideStatus::Running || next == RunbookOverrideStatus::Done;
            }
            RunbookOverrideStatus::Running => {
                match next {
                    RunbookOverrideStatus::Done | RunbookOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookOverrideStatus::Done => {
                return next == RunbookOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook override", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.capacity as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookOverride {
        RunbookOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            tenant_id: self.tenant_id.clone(),
            phone: self.phone.clone(),
            priority: self.priority,
            scope: self.scope.clone(),
            country: self.country.clone(),
            updated_at: self.updated_at,
            score: self.score,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a runbook override in status new.
pub fn new_runbook_override(code: &str, name: &str) -> RunbookOverride {
    RunbookOverride {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        tenant_id: "".to_string(),
        phone: "".to_string(),
        priority: 0,
        scope: "".to_string(),
        country: "".to_string(),
        updated_at: 0,
        score: 0.0,
        pinned: false,
        status: RunbookOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_runbook_override("code-2", "name-323");
        assert_eq!(item.status, RunbookOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_runbook_override("code-2", "name-323");
        if item.move_to(RunbookOverrideStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookOverrideStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_runbook_override("code-2", "name-323");
        item.status = RunbookOverrideStatus::Failed;
        assert!(
            item.move_to(RunbookOverrideStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
