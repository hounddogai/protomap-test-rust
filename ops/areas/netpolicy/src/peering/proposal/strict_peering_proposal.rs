use super::*;

use acme_kit::text;

/// Where a strict peering proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPeeringProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictPeeringProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictPeeringProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict peering proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringProposal {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub summary: String,
    pub fingerprint: String,
    pub maximum: f64,
    pub principal: String,
    pub updated_at: i64,
    pub locale: String,
    pub status: StrictPeeringProposalStatus,
}

impl StrictPeeringProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPeeringProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPeeringProposalStatus) -> bool {
        match self.status {
            StrictPeeringProposalStatus::Created => {
                match next {
                    StrictPeeringProposalStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPeeringProposalStatus::Validated => {
                match next {
                    StrictPeeringProposalStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPeeringProposalStatus::Published => {
                return next == StrictPeeringProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPeeringProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict peering proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        (self.maximum * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPeeringProposal {
        StrictPeeringProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            summary: self.summary.clone(),
            fingerprint: self.fingerprint.clone(),
            maximum: self.maximum,
            principal: self.principal.clone(),
            updated_at: self.updated_at,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict peering proposal in status created.
pub fn new_strict_peering_proposal(code: &str, name: &str) -> StrictPeeringProposal {
    StrictPeeringProposal {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        summary: "".to_string(),
        fingerprint: "".to_string(),
        maximum: 0.0,
        principal: "".to_string(),
        updated_at: 0,
        locale: "".to_string(),
        status: StrictPeeringProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_peering_proposal("code-638", "name-293");
        assert_eq!(item.status, StrictPeeringProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_peering_proposal("code-638", "name-293");
        if item.move_to(StrictPeeringProposalStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPeeringProposalStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_peering_proposal("code-638", "name-293");
        item.status = StrictPeeringProposalStatus::Retired;
        assert!(
            item.move_to(StrictPeeringProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
