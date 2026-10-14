use super::*;

use acme_kit::text;

/// Where a peering proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PeeringProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PeeringProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering proposal as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringProposal {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub maximum: f64,
    pub principal: String,
    pub status: PeeringProposalStatus,
}

impl PeeringProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringProposalStatus) -> bool {
        match self.status {
            PeeringProposalStatus::Draft => {
                match next {
                    PeeringProposalStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringProposalStatus::Pending => {
                match next {
                    PeeringProposalStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringProposalStatus::Active => {
                match next {
                    PeeringProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringProposalStatus::Suspended => {
                return next == PeeringProposalStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "peering proposal", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.maximum * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringProposal {
        PeeringProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            maximum: self.maximum,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a peering proposal in status draft.
pub fn new_peering_proposal(code: &str, title: &str) -> PeeringProposal {
    PeeringProposal {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        maximum: 0.0,
        principal: "".to_string(),
        status: PeeringProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_peering_proposal("code-80", "title-492");
        assert_eq!(item.status, PeeringProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_peering_proposal("code-80", "title-492");
        if item.move_to(PeeringProposalStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringProposalStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_peering_proposal("code-80", "title-492");
        item.status = PeeringProposalStatus::Closed;
        assert!(
            item.move_to(PeeringProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
