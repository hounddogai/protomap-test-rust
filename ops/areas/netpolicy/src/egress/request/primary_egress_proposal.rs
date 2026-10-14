use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary egress proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEgressProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryEgressProposalStatus {
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

impl std::fmt::Display for PrimaryEgressProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary egress proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressProposal {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub attributes: BTreeMap<String, String>,
    pub visible: bool,
    pub status: PrimaryEgressProposalStatus,
}

impl PrimaryEgressProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEgressProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEgressProposalStatus) -> bool {
        match self.status {
            PrimaryEgressProposalStatus::Draft => {
                match next {
                    PrimaryEgressProposalStatus::Active | PrimaryEgressProposalStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEgressProposalStatus::Pending => {
                return next == PrimaryEgressProposalStatus::Active;
            }
            PrimaryEgressProposalStatus::Active => {
                match next {
                    PrimaryEgressProposalStatus::Suspended | PrimaryEgressProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEgressProposalStatus::Suspended => {
                return next == PrimaryEgressProposalStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryEgressProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary egress proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> PrimaryEgressProposal {
        PrimaryEgressProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            attributes: self.attributes.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary egress proposal in status draft.
pub fn new_primary_egress_proposal(code: &str, name: &str) -> PrimaryEgressProposal {
    PrimaryEgressProposal {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        attributes: BTreeMap::new(),
        visible: false,
        status: PrimaryEgressProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_egress_proposal("code-182", "name-983");
        assert_eq!(item.status, PrimaryEgressProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_primary_egress_proposal("code-182", "name-983");
        if item.move_to(PrimaryEgressProposalStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEgressProposalStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_egress_proposal("code-182", "name-983");
        item.status = PrimaryEgressProposalStatus::Closed;
        assert!(
            item.move_to(PrimaryEgressProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
