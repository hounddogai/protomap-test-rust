use super::*;

use acme_kit::text;

/// Where a retention proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionProposalStatus {
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

impl std::fmt::Display for RetentionProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention proposal as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProposal {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub weight: f64,
    pub threshold: f64,
    pub priority: i64,
    pub tenant_id: String,
    pub port: i64,
    pub visible: bool,
    pub external_ref: String,
    pub principal: String,
    pub status: RetentionProposalStatus,
}

impl RetentionProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionProposalStatus) -> bool {
        match self.status {
            RetentionProposalStatus::New => {
                return next == RetentionProposalStatus::Failed;
            }
            RetentionProposalStatus::Queued => {
                match next {
                    RetentionProposalStatus::Done | RetentionProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionProposalStatus::Running => {
                match next {
                    RetentionProposalStatus::Done | RetentionProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionProposalStatus::Done => {
                match next {
                    RetentionProposalStatus::Failed => {
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

    pub fn move_to(&mut self, next: RetentionProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "retention proposal", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.weight + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionProposal {
        RetentionProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            weight: self.weight,
            threshold: self.threshold,
            priority: self.priority,
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            visible: self.visible,
            external_ref: self.external_ref.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a retention proposal in status new.
pub fn new_retention_proposal(code: &str, title: &str) -> RetentionProposal {
    RetentionProposal {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        weight: 0.0,
        threshold: 0.0,
        priority: 0,
        tenant_id: "".to_string(),
        port: 0,
        visible: false,
        external_ref: "".to_string(),
        principal: "".to_string(),
        status: RetentionProposalStatus::New,
    }
}
