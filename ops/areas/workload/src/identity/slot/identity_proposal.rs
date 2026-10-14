use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a identity proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IdentityProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IdentityProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity proposal as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityProposal {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub score: f64,
    pub source: String,
    pub cluster: String,
    pub attributes: BTreeMap<String, String>,
    pub phone: String,
    pub scope: String,
    pub ceiling: f64,
    pub tenant_id: String,
    pub status: IdentityProposalStatus,
}

impl IdentityProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityProposalStatus) -> bool {
        match self.status {
            IdentityProposalStatus::Created => {
                return next == IdentityProposalStatus::Retired;
            }
            IdentityProposalStatus::Validated => {
                return next == IdentityProposalStatus::Published || next == IdentityProposalStatus::Retired;
            }
            IdentityProposalStatus::Published => {
                return next == IdentityProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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

    pub fn effective_value(&self) -> f64 {
        (self.score * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityProposal {
        IdentityProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            score: self.score,
            source: self.source.clone(),
            cluster: self.cluster.clone(),
            attributes: self.attributes.clone(),
            phone: self.phone.clone(),
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a identity proposal in status created.
pub fn new_identity_proposal(code: &str, name: &str) -> IdentityProposal {
    IdentityProposal {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        score: 0.0,
        source: "".to_string(),
        cluster: "".to_string(),
        attributes: BTreeMap::new(),
        phone: "".to_string(),
        scope: "".to_string(),
        ceiling: 0.0,
        tenant_id: "".to_string(),
        status: IdentityProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_identity_proposal("code-590", "name-983");
        assert_eq!(item.status, IdentityProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_identity_proposal("code-590", "name-983");
        if item.move_to(IdentityProposalStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityProposalStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_identity_proposal("code-590", "name-983");
        item.status = IdentityProposalStatus::Retired;
        assert!(
            item.move_to(IdentityProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
