use super::*;

use acme_kit::text;

/// Where a global limit proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalLimitProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalLimitProposalStatus {
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

impl std::fmt::Display for GlobalLimitProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global limit proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitProposal {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub score: f64,
    pub category: String,
    pub ratio: f64,
    pub status: GlobalLimitProposalStatus,
}

impl GlobalLimitProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalLimitProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalLimitProposalStatus) -> bool {
        match self.status {
            GlobalLimitProposalStatus::New => {
                match next {
                    GlobalLimitProposalStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalLimitProposalStatus::Queued => {
                return next == GlobalLimitProposalStatus::Running;
            }
            GlobalLimitProposalStatus::Running => {
                return next == GlobalLimitProposalStatus::Failed;
            }
            GlobalLimitProposalStatus::Done => {
                return next == GlobalLimitProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalLimitProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global limit proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.grace_days as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalLimitProposal {
        GlobalLimitProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            score: self.score,
            category: self.category.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a global limit proposal in status new.
pub fn new_global_limit_proposal(code: &str, name: &str) -> GlobalLimitProposal {
    GlobalLimitProposal {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        score: 0.0,
        category: "".to_string(),
        ratio: 0.0,
        status: GlobalLimitProposalStatus::New,
    }
}
