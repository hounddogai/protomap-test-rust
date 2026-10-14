use super::*;

use acme_kit::text;

/// Where a fallback credential ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackCredentialLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackCredentialLedgerStatus {
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

impl std::fmt::Display for FallbackCredentialLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback credential ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCredentialLedger {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub margin: f64,
    pub expires_at: i64,
    pub quota_units: i64,
    pub tenant_id: String,
    pub status: FallbackCredentialLedgerStatus,
}

impl FallbackCredentialLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackCredentialLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackCredentialLedgerStatus) -> bool {
        match self.status {
            FallbackCredentialLedgerStatus::Open => {
                return next == FallbackCredentialLedgerStatus::InReview
                    || next == FallbackCredentialLedgerStatus::Reopened;
            }
            FallbackCredentialLedgerStatus::InReview => {
                match next {
                    FallbackCredentialLedgerStatus::Resolved | FallbackCredentialLedgerStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackCredentialLedgerStatus::Resolved => {
                match next {
                    FallbackCredentialLedgerStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackCredentialLedgerStatus::Reopened => {
                return next == FallbackCredentialLedgerStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackCredentialLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback credential ledger {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.retries as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackCredentialLedger {
        FallbackCredentialLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            margin: self.margin,
            expires_at: self.expires_at,
            quota_units: self.quota_units,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback credential ledger in status open.
pub fn new_fallback_credential_ledger(code: &str, name: &str) -> FallbackCredentialLedger {
    FallbackCredentialLedger {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        margin: 0.0,
        expires_at: 0,
        quota_units: 0,
        tenant_id: "".to_string(),
        status: FallbackCredentialLedgerStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_credential_ledger("code-87", "name-727");
        assert_eq!(item.status, FallbackCredentialLedgerStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_fallback_credential_ledger("code-87", "name-727");
        if item.move_to(FallbackCredentialLedgerStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackCredentialLedgerStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_credential_ledger("code-87", "name-727");
        item.status = FallbackCredentialLedgerStatus::Archived;
        assert!(
            item.move_to(FallbackCredentialLedgerStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
