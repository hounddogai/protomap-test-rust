use super::*;

use acme_kit::text;

/// Where a manual isolation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualIsolationProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualIsolationProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualIsolationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual isolation proposal as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationProposal {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub active: bool,
    pub external_ref: String,
    pub cost: f64,
    pub notes: String,
    pub key_id: String,
    pub budget: f64,
    pub quantity: i64,
    pub status: ManualIsolationProposalStatus,
}

impl ManualIsolationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualIsolationProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualIsolationProposalStatus) -> bool {
        match self.status {
            ManualIsolationProposalStatus::Requested => {
                match next {
                    ManualIsolationProposalStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIsolationProposalStatus::Granted => {
                match next {
                    ManualIsolationProposalStatus::Revoked | ManualIsolationProposalStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIsolationProposalStatus::Revoked => {
                match next {
                    ManualIsolationProposalStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualIsolationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual isolation proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.cost - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualIsolationProposal {
        ManualIsolationProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            active: self.active,
            external_ref: self.external_ref.clone(),
            cost: self.cost,
            notes: self.notes.clone(),
            key_id: self.key_id.clone(),
            budget: self.budget,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual isolation proposal in status requested.
pub fn new_manual_isolation_proposal(code: &str, name: &str) -> ManualIsolationProposal {
    ManualIsolationProposal {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        active: false,
        external_ref: "".to_string(),
        cost: 0.0,
        notes: "".to_string(),
        key_id: "".to_string(),
        budget: 0.0,
        quantity: 0,
        status: ManualIsolationProposalStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_isolation_proposal("code-367", "name-93");
        assert_eq!(item.status, ManualIsolationProposalStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_manual_isolation_proposal("code-367", "name-93");
        if item.move_to(ManualIsolationProposalStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualIsolationProposalStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_isolation_proposal("code-367", "name-93");
        item.status = ManualIsolationProposalStatus::Expired;
        assert!(
            item.move_to(ManualIsolationProposalStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
