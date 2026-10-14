use super::*;

use acme_kit::text;

/// Where a canary proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CanaryProposalStatus {
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

impl std::fmt::Display for CanaryProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryProposal {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub locale: String,
    pub ratio: f64,
    pub issuer: String,
    pub approved: bool,
    pub status: CanaryProposalStatus,
}

impl CanaryProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryProposalStatus) -> bool {
        match self.status {
            CanaryProposalStatus::New => {
                return next == CanaryProposalStatus::Done || next == CanaryProposalStatus::Failed;
            }
            CanaryProposalStatus::Queued => {
                match next {
                    CanaryProposalStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryProposalStatus::Running => {
                return next == CanaryProposalStatus::Done;
            }
            CanaryProposalStatus::Done => {
                return next == CanaryProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "canary proposal", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.ratio + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryProposal {
        CanaryProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            locale: self.locale.clone(),
            ratio: self.ratio,
            issuer: self.issuer.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a canary proposal in status new.
pub fn new_canary_proposal(code: &str, label: &str) -> CanaryProposal {
    CanaryProposal {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        locale: "".to_string(),
        ratio: 0.0,
        issuer: "".to_string(),
        approved: false,
        status: CanaryProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_canary_proposal("code-733", "label-244");
        assert_eq!(item.status, CanaryProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_canary_proposal("code-733", "label-244");
        if item.move_to(CanaryProposalStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryProposalStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_canary_proposal("code-733", "label-244");
        item.status = CanaryProposalStatus::Failed;
        assert!(
            item.move_to(CanaryProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
