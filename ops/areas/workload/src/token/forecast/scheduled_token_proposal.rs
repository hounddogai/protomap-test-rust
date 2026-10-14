use super::*;

use acme_kit::text;

/// Where a scheduled token proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTokenProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledTokenProposalStatus {
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

impl std::fmt::Display for ScheduledTokenProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled token proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTokenProposal {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub timezone: String,
    pub currency: String,
    pub weight: f64,
    pub pinned: bool,
    pub sequence: i64,
    pub fingerprint: String,
    pub limit: i64,
    pub contact_email: String,
    pub status: ScheduledTokenProposalStatus,
}

impl ScheduledTokenProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTokenProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTokenProposalStatus) -> bool {
        match self.status {
            ScheduledTokenProposalStatus::Idle => {
                return next == ScheduledTokenProposalStatus::Stopped;
            }
            ScheduledTokenProposalStatus::Warming => {
                match next {
                    ScheduledTokenProposalStatus::Ready | ScheduledTokenProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTokenProposalStatus::Ready => {
                return next == ScheduledTokenProposalStatus::Draining;
            }
            ScheduledTokenProposalStatus::Draining => {
                return next == ScheduledTokenProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledTokenProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled token proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.sequence as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTokenProposal {
        ScheduledTokenProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            timezone: self.timezone.clone(),
            currency: self.currency.clone(),
            weight: self.weight,
            pinned: self.pinned,
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            limit: self.limit,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled token proposal in status idle.
pub fn new_scheduled_token_proposal(code: &str, name: &str) -> ScheduledTokenProposal {
    ScheduledTokenProposal {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        timezone: "".to_string(),
        currency: "".to_string(),
        weight: 0.0,
        pinned: false,
        sequence: 0,
        fingerprint: "".to_string(),
        limit: 0,
        contact_email: "".to_string(),
        status: ScheduledTokenProposalStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_token_proposal("code-160", "name-517");
        assert_eq!(item.status, ScheduledTokenProposalStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_scheduled_token_proposal("code-160", "name-517");
        if item.move_to(ScheduledTokenProposalStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTokenProposalStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_token_proposal("code-160", "name-517");
        item.status = ScheduledTokenProposalStatus::Stopped;
        assert!(
            item.move_to(ScheduledTokenProposalStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
