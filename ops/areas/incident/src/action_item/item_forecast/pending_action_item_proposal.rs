use super::*;

use acme_kit::text;

/// Where a pending action item proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingActionItemProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingActionItemProposalStatus {
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

impl std::fmt::Display for PendingActionItemProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending action item proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemProposal {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub maximum: f64,
    pub margin: f64,
    pub quota_units: i64,
    pub revision: i64,
    pub title: String,
    pub summary: String,
    pub rotation_days: i64,
    pub status: PendingActionItemProposalStatus,
}

impl PendingActionItemProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingActionItemProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingActionItemProposalStatus) -> bool {
        match self.status {
            PendingActionItemProposalStatus::New => {
                match next {
                    PendingActionItemProposalStatus::Queued | PendingActionItemProposalStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingActionItemProposalStatus::Queued => {
                return next == PendingActionItemProposalStatus::Running
                    || next == PendingActionItemProposalStatus::Failed;
            }
            PendingActionItemProposalStatus::Running => {
                return next == PendingActionItemProposalStatus::Done
                    || next == PendingActionItemProposalStatus::Failed;
            }
            PendingActionItemProposalStatus::Done => {
                return next == PendingActionItemProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingActionItemProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending action item proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rotation_days as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingActionItemProposal {
        PendingActionItemProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            maximum: self.maximum,
            margin: self.margin,
            quota_units: self.quota_units,
            revision: self.revision,
            title: self.title.clone(),
            summary: self.summary.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending action item proposal in status new.
pub fn new_pending_action_item_proposal(code: &str, name: &str) -> PendingActionItemProposal {
    PendingActionItemProposal {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        maximum: 0.0,
        margin: 0.0,
        quota_units: 0,
        revision: 0,
        title: "".to_string(),
        summary: "".to_string(),
        rotation_days: 0,
        status: PendingActionItemProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_action_item_proposal("code-178", "name-536");
        assert_eq!(item.status, PendingActionItemProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_pending_action_item_proposal("code-178", "name-536");
        if item.move_to(PendingActionItemProposalStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingActionItemProposalStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_action_item_proposal("code-178", "name-536");
        item.status = PendingActionItemProposalStatus::Failed;
        assert!(
            item.move_to(PendingActionItemProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
