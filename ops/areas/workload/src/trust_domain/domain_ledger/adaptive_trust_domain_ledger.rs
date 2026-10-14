use super::*;

use acme_kit::text;

/// Where a adaptive trust domain ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTrustDomainLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveTrustDomainLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveTrustDomainLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive trust domain ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainLedger {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub cost: f64,
    pub checksum: String,
    pub namespace: String,
    pub limit: i64,
    pub active: bool,
    pub status: AdaptiveTrustDomainLedgerStatus,
}

impl AdaptiveTrustDomainLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTrustDomainLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTrustDomainLedgerStatus) -> bool {
        match self.status {
            AdaptiveTrustDomainLedgerStatus::Created => {
                return next == AdaptiveTrustDomainLedgerStatus::Validated
                    || next == AdaptiveTrustDomainLedgerStatus::Retired;
            }
            AdaptiveTrustDomainLedgerStatus::Validated => {
                return next == AdaptiveTrustDomainLedgerStatus::Retired;
            }
            AdaptiveTrustDomainLedgerStatus::Published => {
                return next == AdaptiveTrustDomainLedgerStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveTrustDomainLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive trust domain ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.revision as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveTrustDomainLedger {
        AdaptiveTrustDomainLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            cost: self.cost,
            checksum: self.checksum.clone(),
            namespace: self.namespace.clone(),
            limit: self.limit,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive trust domain ledger in status created.
pub fn new_adaptive_trust_domain_ledger(code: &str, title: &str) -> AdaptiveTrustDomainLedger {
    AdaptiveTrustDomainLedger {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        cost: 0.0,
        checksum: "".to_string(),
        namespace: "".to_string(),
        limit: 0,
        active: false,
        status: AdaptiveTrustDomainLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_trust_domain_ledger("code-877", "title-508");
        assert_eq!(item.status, AdaptiveTrustDomainLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_adaptive_trust_domain_ledger("code-877", "title-508");
        if item.move_to(AdaptiveTrustDomainLedgerStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveTrustDomainLedgerStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_trust_domain_ledger("code-877", "title-508");
        item.status = AdaptiveTrustDomainLedgerStatus::Retired;
        assert!(
            item.move_to(AdaptiveTrustDomainLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
