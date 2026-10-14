use super::*;

use acme_kit::text;

/// Where a weighted escalation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedEscalationProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedEscalationProposalStatus {
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

impl std::fmt::Display for WeightedEscalationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted escalation proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationProposal {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub burst: i64,
    pub host: String,
    pub status: WeightedEscalationProposalStatus,
}

impl WeightedEscalationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedEscalationProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedEscalationProposalStatus) -> bool {
        match self.status {
            WeightedEscalationProposalStatus::Idle => {
                return next == WeightedEscalationProposalStatus::Stopped;
            }
            WeightedEscalationProposalStatus::Warming => {
                return next == WeightedEscalationProposalStatus::Ready;
            }
            WeightedEscalationProposalStatus::Ready => {
                match next {
                    WeightedEscalationProposalStatus::Draining | WeightedEscalationProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedEscalationProposalStatus::Draining => {
                return next == WeightedEscalationProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedEscalationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted escalation proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> WeightedEscalationProposal {
        WeightedEscalationProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            burst: self.burst,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted escalation proposal in status idle.
pub fn new_weighted_escalation_proposal(code: &str, label: &str) -> WeightedEscalationProposal {
    WeightedEscalationProposal {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        burst: 0,
        host: "".to_string(),
        status: WeightedEscalationProposalStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_escalation_proposal("code-582", "label-330");
        assert_eq!(item.status, WeightedEscalationProposalStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_weighted_escalation_proposal("code-582", "label-330");
        if item.move_to(WeightedEscalationProposalStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedEscalationProposalStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_escalation_proposal("code-582", "label-330");
        item.status = WeightedEscalationProposalStatus::Stopped;
        assert!(
            item.move_to(WeightedEscalationProposalStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
