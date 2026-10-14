use super::*;

use acme_kit::text;

/// Where a nonce proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl NonceProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for NonceProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce proposal as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceProposal {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub margin: f64,
    pub summary: String,
    pub cost: f64,
    pub port: i64,
    pub status: NonceProposalStatus,
}

impl NonceProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceProposalStatus) -> bool {
        match self.status {
            NonceProposalStatus::Proposed => {
                return next == NonceProposalStatus::Approved || next == NonceProposalStatus::Rejected;
            }
            NonceProposalStatus::Approved => {
                match next {
                    NonceProposalStatus::Rejected | NonceProposalStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceProposalStatus::Rejected => {
                match next {
                    NonceProposalStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: NonceProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce proposal", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.margin - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceProposal {
        NonceProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            margin: self.margin,
            summary: self.summary.clone(),
            cost: self.cost,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a nonce proposal in status proposed.
pub fn new_nonce_proposal(code: &str, title: &str) -> NonceProposal {
    NonceProposal {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        margin: 0.0,
        summary: "".to_string(),
        cost: 0.0,
        port: 0,
        status: NonceProposalStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_nonce_proposal("code-399", "title-230");
        assert_eq!(item.status, NonceProposalStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_nonce_proposal("code-399", "title-230");
        if item.move_to(NonceProposalStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceProposalStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_nonce_proposal("code-399", "title-230");
        item.status = NonceProposalStatus::Withdrawn;
        assert!(
            item.move_to(NonceProposalStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
