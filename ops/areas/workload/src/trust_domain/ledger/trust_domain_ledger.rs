use super::*;

use acme_kit::text;

/// Where a trust domain ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TrustDomainLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TrustDomainLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainLedger {
    pub code: String,
    pub name: String,
    pub source: String,
    pub tenant_id: String,
    pub cipher: String,
    pub external_ref: String,
    pub locale: String,
    pub attempts: i64,
    pub maximum: f64,
    pub country: String,
    pub status: TrustDomainLedgerStatus,
}

impl TrustDomainLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainLedgerStatus) -> bool {
        match self.status {
            TrustDomainLedgerStatus::Proposed => {
                return next == TrustDomainLedgerStatus::Rejected || next == TrustDomainLedgerStatus::Withdrawn;
            }
            TrustDomainLedgerStatus::Approved => {
                match next {
                    TrustDomainLedgerStatus::Rejected | TrustDomainLedgerStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainLedgerStatus::Rejected => {
                match next {
                    TrustDomainLedgerStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TrustDomainLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "trust domain ledger", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.maximum - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainLedger {
        TrustDomainLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            tenant_id: self.tenant_id.clone(),
            cipher: self.cipher.clone(),
            external_ref: self.external_ref.clone(),
            locale: self.locale.clone(),
            attempts: self.attempts,
            maximum: self.maximum,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a trust domain ledger in status proposed.
pub fn new_trust_domain_ledger(code: &str, name: &str) -> TrustDomainLedger {
    TrustDomainLedger {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        tenant_id: "".to_string(),
        cipher: "".to_string(),
        external_ref: "".to_string(),
        locale: "".to_string(),
        attempts: 0,
        maximum: 0.0,
        country: "".to_string(),
        status: TrustDomainLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_trust_domain_ledger("code-531", "name-799");
        assert_eq!(item.status, TrustDomainLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_trust_domain_ledger("code-531", "name-799");
        if item.move_to(TrustDomainLedgerStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainLedgerStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_trust_domain_ledger("code-531", "name-799");
        item.status = TrustDomainLedgerStatus::Withdrawn;
        assert!(
            item.move_to(TrustDomainLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
