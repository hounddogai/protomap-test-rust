use super::*;

use acme_kit::text;

/// Where a lenient egress proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEgressProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientEgressProposalStatus {
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

impl std::fmt::Display for LenientEgressProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient egress proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressProposal {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub floor_value: f64,
    pub phone: String,
    pub status: LenientEgressProposalStatus,
}

impl LenientEgressProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEgressProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEgressProposalStatus) -> bool {
        match self.status {
            LenientEgressProposalStatus::New => {
                match next {
                    LenientEgressProposalStatus::Running | LenientEgressProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressProposalStatus::Queued => {
                return next == LenientEgressProposalStatus::Running || next == LenientEgressProposalStatus::Failed;
            }
            LenientEgressProposalStatus::Running => {
                match next {
                    LenientEgressProposalStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressProposalStatus::Done => {
                return next == LenientEgressProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientEgressProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient egress proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.floor_value - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientEgressProposal {
        LenientEgressProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            floor_value: self.floor_value,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient egress proposal in status new.
pub fn new_lenient_egress_proposal(code: &str, name: &str) -> LenientEgressProposal {
    LenientEgressProposal {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        floor_value: 0.0,
        phone: "".to_string(),
        status: LenientEgressProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_egress_proposal("code-425", "name-862");
        assert_eq!(item.status, LenientEgressProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_lenient_egress_proposal("code-425", "name-862");
        if item.move_to(LenientEgressProposalStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientEgressProposalStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_egress_proposal("code-425", "name-862");
        item.status = LenientEgressProposalStatus::Failed;
        assert!(
            item.move_to(LenientEgressProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
