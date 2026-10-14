use super::*;

use acme_kit::text;

/// Where a manual nonce proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualNonceProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualNonceProposalStatus {
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

impl std::fmt::Display for ManualNonceProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual nonce proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceProposal {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub updated_at: i64,
    pub created_at: i64,
    pub weight: f64,
    pub fingerprint: String,
    pub category: String,
    pub tenant_id: String,
    pub attempts: i64,
    pub margin: f64,
    pub status: ManualNonceProposalStatus,
}

impl ManualNonceProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualNonceProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualNonceProposalStatus) -> bool {
        match self.status {
            ManualNonceProposalStatus::New => {
                return next == ManualNonceProposalStatus::Running;
            }
            ManualNonceProposalStatus::Queued => {
                match next {
                    ManualNonceProposalStatus::Running | ManualNonceProposalStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualNonceProposalStatus::Running => {
                match next {
                    ManualNonceProposalStatus::Done | ManualNonceProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualNonceProposalStatus::Done => {
                return next == ManualNonceProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualNonceProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual nonce proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.updated_at as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualNonceProposal {
        ManualNonceProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            updated_at: self.updated_at,
            created_at: self.created_at,
            weight: self.weight,
            fingerprint: self.fingerprint.clone(),
            category: self.category.clone(),
            tenant_id: self.tenant_id.clone(),
            attempts: self.attempts,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a manual nonce proposal in status new.
pub fn new_manual_nonce_proposal(code: &str, name: &str) -> ManualNonceProposal {
    ManualNonceProposal {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        updated_at: 0,
        created_at: 0,
        weight: 0.0,
        fingerprint: "".to_string(),
        category: "".to_string(),
        tenant_id: "".to_string(),
        attempts: 0,
        margin: 0.0,
        status: ManualNonceProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_nonce_proposal("code-295", "name-547");
        assert_eq!(item.status, ManualNonceProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_manual_nonce_proposal("code-295", "name-547");
        if item.move_to(ManualNonceProposalStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualNonceProposalStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_nonce_proposal("code-295", "name-547");
        item.status = ManualNonceProposalStatus::Failed;
        assert!(
            item.move_to(ManualNonceProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
