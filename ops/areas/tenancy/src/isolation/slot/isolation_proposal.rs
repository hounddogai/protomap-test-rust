use super::*;

use acme_kit::text;

/// Where a isolation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl IsolationProposalStatus {
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

impl std::fmt::Display for IsolationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProposal {
    pub code: String,
    pub title: String,
    pub source: String,
    pub currency: String,
    pub batch_size: i64,
    pub port: i64,
    pub archived: bool,
    pub key_id: String,
    pub status: IsolationProposalStatus,
}

impl IsolationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationProposalStatus) -> bool {
        match self.status {
            IsolationProposalStatus::New => {
                return next == IsolationProposalStatus::Running;
            }
            IsolationProposalStatus::Queued => {
                return next == IsolationProposalStatus::Running || next == IsolationProposalStatus::Done;
            }
            IsolationProposalStatus::Running => {
                match next {
                    IsolationProposalStatus::Done | IsolationProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationProposalStatus::Done => {
                match next {
                    IsolationProposalStatus::Failed => {
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

    pub fn move_to(&mut self, next: IsolationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.batch_size as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationProposal {
        IsolationProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            currency: self.currency.clone(),
            batch_size: self.batch_size,
            port: self.port,
            archived: self.archived,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a isolation proposal in status new.
pub fn new_isolation_proposal(code: &str, title: &str) -> IsolationProposal {
    IsolationProposal {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        currency: "".to_string(),
        batch_size: 0,
        port: 0,
        archived: false,
        key_id: "".to_string(),
        status: IsolationProposalStatus::New,
    }
}
