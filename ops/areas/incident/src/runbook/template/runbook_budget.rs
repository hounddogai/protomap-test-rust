use super::*;

use acme_kit::text;

/// Where a runbook budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RunbookBudgetStatus {
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

impl std::fmt::Display for RunbookBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBudget {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub key_id: String,
    pub region: String,
    pub approved: bool,
    pub updated_at: i64,
    pub tenant_id: String,
    pub checksum: String,
    pub created_at: i64,
    pub revision: i64,
    pub status: RunbookBudgetStatus,
}

impl RunbookBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookBudgetStatus) -> bool {
        match self.status {
            RunbookBudgetStatus::New => {
                match next {
                    RunbookBudgetStatus::Queued | RunbookBudgetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookBudgetStatus::Queued => {
                match next {
                    RunbookBudgetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookBudgetStatus::Running => {
                return next == RunbookBudgetStatus::Done || next == RunbookBudgetStatus::Failed;
            }
            RunbookBudgetStatus::Done => {
                match next {
                    RunbookBudgetStatus::Failed => {
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

    pub fn move_to(&mut self, next: RunbookBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.updated_at as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookBudget {
        RunbookBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            key_id: self.key_id.clone(),
            region: self.region.clone(),
            approved: self.approved,
            updated_at: self.updated_at,
            tenant_id: self.tenant_id.clone(),
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a runbook budget in status new.
pub fn new_runbook_budget(code: &str, title: &str) -> RunbookBudget {
    RunbookBudget {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        key_id: "".to_string(),
        region: "".to_string(),
        approved: false,
        updated_at: 0,
        tenant_id: "".to_string(),
        checksum: "".to_string(),
        created_at: 0,
        revision: 0,
        status: RunbookBudgetStatus::New,
    }
}
