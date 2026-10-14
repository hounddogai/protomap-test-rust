use super::*;

use acme_kit::text;

/// Where a runbook proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RunbookProposalStatus {
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

impl std::fmt::Display for RunbookProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook proposal as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookProposal {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub maximum: f64,
    pub minimum: f64,
    pub status: RunbookProposalStatus,
}

impl RunbookProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookProposalStatus) -> bool {
        match self.status {
            RunbookProposalStatus::New => {
                return next == RunbookProposalStatus::Failed;
            }
            RunbookProposalStatus::Queued => {
                match next {
                    RunbookProposalStatus::Running | RunbookProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookProposalStatus::Running => {
                match next {
                    RunbookProposalStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookProposalStatus::Done => {
                return next == RunbookProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.priority as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookProposal {
        RunbookProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            maximum: self.maximum,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a runbook proposal in status new.
pub fn new_runbook_proposal(code: &str, label: &str) -> RunbookProposal {
    RunbookProposal {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        maximum: 0.0,
        minimum: 0.0,
        status: RunbookProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_runbook_proposal("code-907", "label-681");
        assert_eq!(item.status, RunbookProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_runbook_proposal("code-907", "label-681");
        if item.move_to(RunbookProposalStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookProposalStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_runbook_proposal("code-907", "label-681");
        item.status = RunbookProposalStatus::Failed;
        assert!(
            item.move_to(RunbookProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
