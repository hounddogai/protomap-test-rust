use super::*;

use acme_kit::text;

/// Where a runbook checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RunbookCheckpointStatus {
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

impl std::fmt::Display for RunbookCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookCheckpoint {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub external_ref: String,
    pub cluster: String,
    pub currency: String,
    pub country: String,
    pub approved: bool,
    pub weight: f64,
    pub retries: i64,
    pub status: RunbookCheckpointStatus,
}

impl RunbookCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookCheckpointStatus) -> bool {
        match self.status {
            RunbookCheckpointStatus::New => {
                return next == RunbookCheckpointStatus::Queued;
            }
            RunbookCheckpointStatus::Queued => {
                return next == RunbookCheckpointStatus::Running;
            }
            RunbookCheckpointStatus::Running => {
                match next {
                    RunbookCheckpointStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookCheckpointStatus::Done => {
                match next {
                    RunbookCheckpointStatus::Failed => {
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

    pub fn move_to(&mut self, next: RunbookCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.weight - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookCheckpoint {
        RunbookCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            external_ref: self.external_ref.clone(),
            cluster: self.cluster.clone(),
            currency: self.currency.clone(),
            country: self.country.clone(),
            approved: self.approved,
            weight: self.weight,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a runbook checkpoint in status new.
pub fn new_runbook_checkpoint(code: &str, label: &str) -> RunbookCheckpoint {
    RunbookCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        external_ref: "".to_string(),
        cluster: "".to_string(),
        currency: "".to_string(),
        country: "".to_string(),
        approved: false,
        weight: 0.0,
        retries: 0,
        status: RunbookCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_runbook_checkpoint("code-689", "label-236");
        assert_eq!(item.status, RunbookCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_runbook_checkpoint("code-689", "label-236");
        if item.move_to(RunbookCheckpointStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookCheckpointStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_runbook_checkpoint("code-689", "label-236");
        item.status = RunbookCheckpointStatus::Failed;
        assert!(
            item.move_to(RunbookCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
