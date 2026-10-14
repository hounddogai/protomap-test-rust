use super::*;

use acme_kit::text;

/// Where a legacy failover proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFailoverProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyFailoverProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyFailoverProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy failover proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverProposal {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub created_at: i64,
    pub capacity: i64,
    pub scope: String,
    pub retries: i64,
    pub port: i64,
    pub minimum: f64,
    pub threshold: f64,
    pub unit: String,
    pub status: LegacyFailoverProposalStatus,
}

impl LegacyFailoverProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFailoverProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFailoverProposalStatus) -> bool {
        match self.status {
            LegacyFailoverProposalStatus::Proposed => {
                match next {
                    LegacyFailoverProposalStatus::Approved | LegacyFailoverProposalStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFailoverProposalStatus::Approved => {
                return next == LegacyFailoverProposalStatus::Rejected
                    || next == LegacyFailoverProposalStatus::Withdrawn;
            }
            LegacyFailoverProposalStatus::Rejected => {
                match next {
                    LegacyFailoverProposalStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacyFailoverProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy failover proposal {} cannot move from {} to {}",
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
        format!("{} {}", "legacy failover proposal", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.minimum - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFailoverProposal {
        LegacyFailoverProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            created_at: self.created_at,
            capacity: self.capacity,
            scope: self.scope.clone(),
            retries: self.retries,
            port: self.port,
            minimum: self.minimum,
            threshold: self.threshold,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy failover proposal in status proposed.
pub fn new_legacy_failover_proposal(code: &str, title: &str) -> LegacyFailoverProposal {
    LegacyFailoverProposal {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        created_at: 0,
        capacity: 0,
        scope: "".to_string(),
        retries: 0,
        port: 0,
        minimum: 0.0,
        threshold: 0.0,
        unit: "".to_string(),
        status: LegacyFailoverProposalStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_failover_proposal("code-297", "title-589");
        assert_eq!(item.status, LegacyFailoverProposalStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_legacy_failover_proposal("code-297", "title-589");
        if item.move_to(LegacyFailoverProposalStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFailoverProposalStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_failover_proposal("code-297", "title-589");
        item.status = LegacyFailoverProposalStatus::Withdrawn;
        assert!(
            item.move_to(LegacyFailoverProposalStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
