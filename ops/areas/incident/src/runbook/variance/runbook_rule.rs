use super::*;

use acme_kit::text;

/// Where a runbook rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RunbookRuleStatus {
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

impl std::fmt::Display for RunbookRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookRule {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub host: String,
    pub priority: i64,
    pub attempts: i64,
    pub cost: f64,
    pub retries: i64,
    pub principal: String,
    pub status: RunbookRuleStatus,
}

impl RunbookRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookRuleStatus) -> bool {
        match self.status {
            RunbookRuleStatus::New => {
                match next {
                    RunbookRuleStatus::Done | RunbookRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookRuleStatus::Queued => {
                match next {
                    RunbookRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookRuleStatus::Running => {
                return next == RunbookRuleStatus::Failed;
            }
            RunbookRuleStatus::Done => {
                return next == RunbookRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.priority as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookRule {
        RunbookRule {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            host: self.host.clone(),
            priority: self.priority,
            attempts: self.attempts,
            cost: self.cost,
            retries: self.retries,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a runbook rule in status new.
pub fn new_runbook_rule(code: &str, title: &str) -> RunbookRule {
    RunbookRule {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        host: "".to_string(),
        priority: 0,
        attempts: 0,
        cost: 0.0,
        retries: 0,
        principal: "".to_string(),
        status: RunbookRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_runbook_rule("code-268", "title-139");
        assert_eq!(item.status, RunbookRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_runbook_rule("code-268", "title-139");
        if item.move_to(RunbookRuleStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookRuleStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_runbook_rule("code-268", "title-139");
        item.status = RunbookRuleStatus::Failed;
        assert!(item.move_to(RunbookRuleStatus::New).is_err(), "expected move to fail");
    }
}
