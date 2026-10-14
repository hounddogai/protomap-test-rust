use super::*;

use acme_kit::text;

/// Where a action item proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ActionItemProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ActionItemProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemProposal {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub sequence: i64,
    pub category: String,
    pub contact_email: String,
    pub port: i64,
    pub limit: i64,
    pub rate: f64,
    pub cost: f64,
    pub status: ActionItemProposalStatus,
}

impl ActionItemProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemProposalStatus) -> bool {
        match self.status {
            ActionItemProposalStatus::Open => {
                match next {
                    ActionItemProposalStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemProposalStatus::InReview => {
                match next {
                    ActionItemProposalStatus::Resolved | ActionItemProposalStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemProposalStatus::Resolved => {
                return next == ActionItemProposalStatus::Reopened || next == ActionItemProposalStatus::Archived;
            }
            ActionItemProposalStatus::Reopened => {
                match next {
                    ActionItemProposalStatus::Archived => {
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

    pub fn move_to(&mut self, next: ActionItemProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.port as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemProposal {
        ActionItemProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            sequence: self.sequence,
            category: self.category.clone(),
            contact_email: self.contact_email.clone(),
            port: self.port,
            limit: self.limit,
            rate: self.rate,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a action item proposal in status open.
pub fn new_action_item_proposal(code: &str, name: &str) -> ActionItemProposal {
    ActionItemProposal {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        sequence: 0,
        category: "".to_string(),
        contact_email: "".to_string(),
        port: 0,
        limit: 0,
        rate: 0.0,
        cost: 0.0,
        status: ActionItemProposalStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_action_item_proposal("code-988", "name-504");
        assert_eq!(item.status, ActionItemProposalStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_action_item_proposal("code-988", "name-504");
        if item.move_to(ActionItemProposalStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemProposalStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_action_item_proposal("code-988", "name-504");
        item.status = ActionItemProposalStatus::Archived;
        assert!(
            item.move_to(ActionItemProposalStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
