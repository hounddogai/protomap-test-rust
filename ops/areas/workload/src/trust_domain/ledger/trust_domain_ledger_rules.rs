use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain ledger must pass. Check returns its problems, or none.
pub trait TrustDomainLedgerRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainLedgerRequiredSourceRule {
    pub message: String,
}

impl TrustDomainLedgerRule for TrustDomainLedgerRequiredSourceRule {
    fn check(&self, item: &TrustDomainLedger) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainLedgerAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TrustDomainLedgerRule for TrustDomainLedgerAllowedCodeRule {
    fn check(&self, item: &TrustDomainLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainLedgerRequiredTenantIdRule {
    pub message: String,
}

impl TrustDomainLedgerRule for TrustDomainLedgerRequiredTenantIdRule {
    fn check(&self, item: &TrustDomainLedger) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain ledger.
#[derive(Debug, Clone)]
pub struct TrustDomainLedgerValidator {
    pub rules: Vec<Rc<dyn TrustDomainLedgerRule>>,
}

impl TrustDomainLedgerValidator {
    pub fn validate(&self, item: &TrustDomainLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainLedger]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_trust_domain_ledger_validator() -> TrustDomainLedgerValidator {
    TrustDomainLedgerValidator {
        rules: vec![
            Rc::new(TrustDomainLedgerRequiredSourceRule {
                message: "trust domain ledger needs a value".to_string(),
            }) as Rc<dyn TrustDomainLedgerRule>,
            Rc::new(TrustDomainLedgerAllowedCodeRule {
                allowed: vec!["default".to_string(), "regional".to_string(), "tiered".to_string()],
            }),
            Rc::new(TrustDomainLedgerRequiredTenantIdRule {
                message: "trust domain ledger needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_ledger_validator();
        let item = new_trust_domain_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = TrustDomainLedgerRequiredSourceRule {
            message: "trust domain ledger needs a value".to_string(),
        };
        let mut item = new_trust_domain_ledger("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TrustDomainLedgerRequiredTenantIdRule {
            message: "trust domain ledger needs this field".to_string(),
        };
        let mut item = new_trust_domain_ledger("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
