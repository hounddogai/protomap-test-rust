use super::*;

use acme_kit::text;

/// Where a strict canary proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictCanaryProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictCanaryProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for StrictCanaryProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict canary proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryProposal {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub retries: i64,
    pub amount: f64,
    pub contact_email: String,
    pub status: StrictCanaryProposalStatus,
}

impl StrictCanaryProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictCanaryProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictCanaryProposalStatus) -> bool {
        match self.status {
            StrictCanaryProposalStatus::Idle => {
                match next {
                    StrictCanaryProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictCanaryProposalStatus::Warming => {
                return next == StrictCanaryProposalStatus::Stopped;
            }
            StrictCanaryProposalStatus::Ready => {
                return next == StrictCanaryProposalStatus::Draining || next == StrictCanaryProposalStatus::Stopped;
            }
            StrictCanaryProposalStatus::Draining => {
                return next == StrictCanaryProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictCanaryProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict canary proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.amount - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictCanaryProposal {
        StrictCanaryProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            retries: self.retries,
            amount: self.amount,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict canary proposal in status idle.
pub fn new_strict_canary_proposal(code: &str, title: &str) -> StrictCanaryProposal {
    StrictCanaryProposal {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        retries: 0,
        amount: 0.0,
        contact_email: "".to_string(),
        status: StrictCanaryProposalStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_canary_proposal("code-764", "title-460");
        assert_eq!(item.status, StrictCanaryProposalStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_strict_canary_proposal("code-764", "title-460");
        if item.move_to(StrictCanaryProposalStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictCanaryProposalStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_canary_proposal("code-764", "title-460");
        item.status = StrictCanaryProposalStatus::Stopped;
        assert!(
            item.move_to(StrictCanaryProposalStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
