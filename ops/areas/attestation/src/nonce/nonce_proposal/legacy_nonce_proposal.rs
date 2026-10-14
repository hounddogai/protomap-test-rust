use super::*;

use acme_kit::text;

/// Where a legacy nonce proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyNonceProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyNonceProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyNonceProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy nonce proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceProposal {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub updated_at: i64,
    pub cipher: String,
    pub score: f64,
    pub name: String,
    pub fingerprint: String,
    pub status: LegacyNonceProposalStatus,
}

impl LegacyNonceProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyNonceProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyNonceProposalStatus) -> bool {
        match self.status {
            LegacyNonceProposalStatus::Requested => {
                return next == LegacyNonceProposalStatus::Granted;
            }
            LegacyNonceProposalStatus::Granted => {
                return next == LegacyNonceProposalStatus::Expired;
            }
            LegacyNonceProposalStatus::Revoked => {
                return next == LegacyNonceProposalStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyNonceProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy nonce proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy nonce proposal", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.updated_at as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyNonceProposal {
        LegacyNonceProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            updated_at: self.updated_at,
            cipher: self.cipher.clone(),
            score: self.score,
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a legacy nonce proposal in status requested.
pub fn new_legacy_nonce_proposal(code: &str, label: &str) -> LegacyNonceProposal {
    LegacyNonceProposal {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        updated_at: 0,
        cipher: "".to_string(),
        score: 0.0,
        name: "".to_string(),
        fingerprint: "".to_string(),
        status: LegacyNonceProposalStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_nonce_proposal("code-362", "label-137");
        assert_eq!(item.status, LegacyNonceProposalStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_legacy_nonce_proposal("code-362", "label-137");
        if item.move_to(LegacyNonceProposalStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyNonceProposalStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_nonce_proposal("code-362", "label-137");
        item.status = LegacyNonceProposalStatus::Expired;
        assert!(
            item.move_to(LegacyNonceProposalStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
