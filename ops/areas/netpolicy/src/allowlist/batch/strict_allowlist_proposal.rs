use super::*;

use acme_kit::text;

/// Where a strict allowlist proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAllowlistProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictAllowlistProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for StrictAllowlistProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict allowlist proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistProposal {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub checksum: String,
    pub external_ref: String,
    pub notes: String,
    pub audience: String,
    pub status: StrictAllowlistProposalStatus,
}

impl StrictAllowlistProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAllowlistProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAllowlistProposalStatus) -> bool {
        match self.status {
            StrictAllowlistProposalStatus::Planned => {
                return next == StrictAllowlistProposalStatus::Cancelled;
            }
            StrictAllowlistProposalStatus::Scheduled => {
                return next == StrictAllowlistProposalStatus::InProgress
                    || next == StrictAllowlistProposalStatus::Completed;
            }
            StrictAllowlistProposalStatus::InProgress => {
                match next {
                    StrictAllowlistProposalStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAllowlistProposalStatus::Completed => {
                match next {
                    StrictAllowlistProposalStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: StrictAllowlistProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict allowlist proposal {} cannot move from {} to {}",
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
        self.maximum
    }

    pub fn copy(&self) -> StrictAllowlistProposal {
        StrictAllowlistProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            checksum: self.checksum.clone(),
            external_ref: self.external_ref.clone(),
            notes: self.notes.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict allowlist proposal in status planned.
pub fn new_strict_allowlist_proposal(code: &str, label: &str) -> StrictAllowlistProposal {
    StrictAllowlistProposal {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        checksum: "".to_string(),
        external_ref: "".to_string(),
        notes: "".to_string(),
        audience: "".to_string(),
        status: StrictAllowlistProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_strict_allowlist_proposal("code-837", "label-74");
        assert_eq!(item.status, StrictAllowlistProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_strict_allowlist_proposal("code-837", "label-74");
        if item.move_to(StrictAllowlistProposalStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictAllowlistProposalStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_strict_allowlist_proposal("code-837", "label-74");
        item.status = StrictAllowlistProposalStatus::Cancelled;
        assert!(
            item.move_to(StrictAllowlistProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
