use super::*;

use acme_kit::text;

/// Where a rollout proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RolloutProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RolloutProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollout proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProposal {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub unit: String,
    pub margin: f64,
    pub category: String,
    pub status: RolloutProposalStatus,
}

impl RolloutProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutProposalStatus) -> bool {
        match self.status {
            RolloutProposalStatus::Created => {
                match next {
                    RolloutProposalStatus::Published | RolloutProposalStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutProposalStatus::Validated => {
                match next {
                    RolloutProposalStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutProposalStatus::Published => {
                return next == RolloutProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> RolloutProposal {
        RolloutProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            unit: self.unit.clone(),
            margin: self.margin,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rollout proposal in status created.
pub fn new_rollout_proposal(code: &str, name: &str) -> RolloutProposal {
    RolloutProposal {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        unit: "".to_string(),
        margin: 0.0,
        category: "".to_string(),
        status: RolloutProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rollout_proposal("code-860", "name-359");
        assert_eq!(item.status, RolloutProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_rollout_proposal("code-860", "name-359");
        if item.move_to(RolloutProposalStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutProposalStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rollout_proposal("code-860", "name-359");
        item.status = RolloutProposalStatus::Retired;
        assert!(
            item.move_to(RolloutProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
