use super::*;

use acme_kit::text;

/// Where a escalation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EscalationProposalStatus {
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

impl std::fmt::Display for EscalationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One escalation proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProposal {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub archived: bool,
    pub ttl_seconds: i64,
    pub scope: String,
    pub visible: bool,
    pub status: EscalationProposalStatus,
}

impl EscalationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationProposalStatus) -> bool {
        match self.status {
            EscalationProposalStatus::New => {
                return next == EscalationProposalStatus::Done || next == EscalationProposalStatus::Failed;
            }
            EscalationProposalStatus::Queued => {
                return next == EscalationProposalStatus::Running;
            }
            EscalationProposalStatus::Running => {
                match next {
                    EscalationProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationProposalStatus::Done => {
                match next {
                    EscalationProposalStatus::Failed => {
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

    pub fn move_to(&mut self, next: EscalationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationProposal {
        EscalationProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            archived: self.archived,
            ttl_seconds: self.ttl_seconds,
            scope: self.scope.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a escalation proposal in status new.
pub fn new_escalation_proposal(code: &str, title: &str) -> EscalationProposal {
    EscalationProposal {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        archived: false,
        ttl_seconds: 0,
        scope: "".to_string(),
        visible: false,
        status: EscalationProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_escalation_proposal("code-667", "title-232");
        assert_eq!(item.status, EscalationProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_escalation_proposal("code-667", "title-232");
        if item.move_to(EscalationProposalStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationProposalStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_escalation_proposal("code-667", "title-232");
        item.status = EscalationProposalStatus::Failed;
        assert!(
            item.move_to(EscalationProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
