use super::*;

use acme_kit::text;

/// Where a trust domain budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TrustDomainBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TrustDomainBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain budget as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBudget {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub archived: bool,
    pub checksum: String,
    pub amount: f64,
    pub status: TrustDomainBudgetStatus,
}

impl TrustDomainBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainBudgetStatus) -> bool {
        match self.status {
            TrustDomainBudgetStatus::Created => {
                return next == TrustDomainBudgetStatus::Validated || next == TrustDomainBudgetStatus::Published;
            }
            TrustDomainBudgetStatus::Validated => {
                return next == TrustDomainBudgetStatus::Retired;
            }
            TrustDomainBudgetStatus::Published => {
                match next {
                    TrustDomainBudgetStatus::Retired => {
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

    pub fn move_to(&mut self, next: TrustDomainBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> TrustDomainBudget {
        TrustDomainBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            archived: self.archived,
            checksum: self.checksum.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a trust domain budget in status created.
pub fn new_trust_domain_budget(code: &str, name: &str) -> TrustDomainBudget {
    TrustDomainBudget {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        archived: false,
        checksum: "".to_string(),
        amount: 0.0,
        status: TrustDomainBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_trust_domain_budget("code-762", "name-52");
        assert_eq!(item.status, TrustDomainBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_trust_domain_budget("code-762", "name-52");
        if item.move_to(TrustDomainBudgetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainBudgetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_trust_domain_budget("code-762", "name-52");
        item.status = TrustDomainBudgetStatus::Retired;
        assert!(
            item.move_to(TrustDomainBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
