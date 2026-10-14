use super::*;

use acme_kit::text;

/// Where a delegation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DelegationProposalStatus {
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

impl std::fmt::Display for DelegationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProposal {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub priority: i64,
    pub issuer: String,
    pub timezone: String,
    pub score: f64,
    pub status: DelegationProposalStatus,
}

impl DelegationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationProposalStatus) -> bool {
        match self.status {
            DelegationProposalStatus::New => {
                match next {
                    DelegationProposalStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationProposalStatus::Queued => {
                return next == DelegationProposalStatus::Running || next == DelegationProposalStatus::Failed;
            }
            DelegationProposalStatus::Running => {
                return next == DelegationProposalStatus::Failed;
            }
            DelegationProposalStatus::Done => {
                match next {
                    DelegationProposalStatus::Failed => {
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

    pub fn move_to(&mut self, next: DelegationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.grace_days as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationProposal {
        DelegationProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            priority: self.priority,
            issuer: self.issuer.clone(),
            timezone: self.timezone.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation proposal in status new.
pub fn new_delegation_proposal(code: &str, label: &str) -> DelegationProposal {
    DelegationProposal {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        priority: 0,
        issuer: "".to_string(),
        timezone: "".to_string(),
        score: 0.0,
        status: DelegationProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_delegation_proposal("code-177", "label-503");
        assert_eq!(item.status, DelegationProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_delegation_proposal("code-177", "label-503");
        if item.move_to(DelegationProposalStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationProposalStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_delegation_proposal("code-177", "label-503");
        item.status = DelegationProposalStatus::Failed;
        assert!(
            item.move_to(DelegationProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
