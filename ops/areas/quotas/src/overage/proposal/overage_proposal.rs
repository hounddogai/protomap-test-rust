use super::*;

use acme_kit::text;

/// Where a overage proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl OverageProposalStatus {
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

impl std::fmt::Display for OverageProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageProposal {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub visible: bool,
    pub key_id: String,
    pub budget: f64,
    pub name: String,
    pub weight: f64,
    pub margin: f64,
    pub revision: i64,
    pub duration_minutes: i64,
    pub status: OverageProposalStatus,
}

impl OverageProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageProposalStatus) -> bool {
        match self.status {
            OverageProposalStatus::Idle => {
                return next == OverageProposalStatus::Ready;
            }
            OverageProposalStatus::Warming => {
                match next {
                    OverageProposalStatus::Draining | OverageProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageProposalStatus::Ready => {
                return next == OverageProposalStatus::Draining || next == OverageProposalStatus::Stopped;
            }
            OverageProposalStatus::Draining => {
                match next {
                    OverageProposalStatus::Stopped => {
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

    pub fn move_to(&mut self, next: OverageProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.duration_minutes as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageProposal {
        OverageProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            visible: self.visible,
            key_id: self.key_id.clone(),
            budget: self.budget,
            name: self.name.clone(),
            weight: self.weight,
            margin: self.margin,
            revision: self.revision,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a overage proposal in status idle.
pub fn new_overage_proposal(code: &str, label: &str) -> OverageProposal {
    OverageProposal {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        visible: false,
        key_id: "".to_string(),
        budget: 0.0,
        name: "".to_string(),
        weight: 0.0,
        margin: 0.0,
        revision: 0,
        duration_minutes: 0,
        status: OverageProposalStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_overage_proposal("code-131", "label-193");
        assert_eq!(item.status, OverageProposalStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_overage_proposal("code-131", "label-193");
        if item.move_to(OverageProposalStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageProposalStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_overage_proposal("code-131", "label-193");
        item.status = OverageProposalStatus::Stopped;
        assert!(
            item.move_to(OverageProposalStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
