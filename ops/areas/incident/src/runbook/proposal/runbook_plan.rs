use super::*;

use acme_kit::text;

/// Where a runbook plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RunbookPlanStatus {
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

impl std::fmt::Display for RunbookPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookPlan {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub unit: String,
    pub source: String,
    pub status: RunbookPlanStatus,
}

impl RunbookPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookPlanStatus) -> bool {
        match self.status {
            RunbookPlanStatus::New => {
                return next == RunbookPlanStatus::Queued || next == RunbookPlanStatus::Running;
            }
            RunbookPlanStatus::Queued => {
                match next {
                    RunbookPlanStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookPlanStatus::Running => {
                match next {
                    RunbookPlanStatus::Done | RunbookPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookPlanStatus::Done => {
                return next == RunbookPlanStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook plan {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> RunbookPlan {
        RunbookPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            unit: self.unit.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a runbook plan in status new.
pub fn new_runbook_plan(code: &str, label: &str) -> RunbookPlan {
    RunbookPlan {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        unit: "".to_string(),
        source: "".to_string(),
        status: RunbookPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_runbook_plan("code-778", "label-700");
        assert_eq!(item.status, RunbookPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_runbook_plan("code-778", "label-700");
        if item.move_to(RunbookPlanStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookPlanStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_runbook_plan("code-778", "label-700");
        item.status = RunbookPlanStatus::Failed;
        assert!(item.move_to(RunbookPlanStatus::New).is_err(), "expected move to fail");
    }
}
