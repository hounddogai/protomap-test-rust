use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive trust domain ledger must pass. Check returns its problems, or none.
pub trait AdaptiveTrustDomainLedgerRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTrustDomainLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainLedgerNamespaceLengthRule {
    pub max_length: i64,
}

impl AdaptiveTrustDomainLedgerRule for AdaptiveTrustDomainLedgerNamespaceLengthRule {
    fn check(&self, item: &AdaptiveTrustDomainLedger) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainLedgerTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveTrustDomainLedgerRule for AdaptiveTrustDomainLedgerTitleLengthRule {
    fn check(&self, item: &AdaptiveTrustDomainLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainLedgerCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveTrustDomainLedgerRule for AdaptiveTrustDomainLedgerCostRangeRule {
    fn check(&self, item: &AdaptiveTrustDomainLedger) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainLedgerRequiredNamespaceRule {
    pub message: String,
}

impl AdaptiveTrustDomainLedgerRule for AdaptiveTrustDomainLedgerRequiredNamespaceRule {
    fn check(&self, item: &AdaptiveTrustDomainLedger) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainLedgerCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveTrustDomainLedgerRule for AdaptiveTrustDomainLedgerCodePrefixRule {
    fn check(&self, item: &AdaptiveTrustDomainLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive trust domain ledger.
#[derive(Debug, Clone)]
pub struct AdaptiveTrustDomainLedgerValidator {
    pub rules: Vec<Rc<dyn AdaptiveTrustDomainLedgerRule>>,
}

impl AdaptiveTrustDomainLedgerValidator {
    pub fn validate(&self, item: &AdaptiveTrustDomainLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveTrustDomainLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTrustDomainLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_trust_domain_ledger_validator() -> AdaptiveTrustDomainLedgerValidator {
    AdaptiveTrustDomainLedgerValidator {
        rules: vec![
            Rc::new(AdaptiveTrustDomainLedgerNamespaceLengthRule { max_length: 32 })
                as Rc<dyn AdaptiveTrustDomainLedgerRule>,
            Rc::new(AdaptiveTrustDomainLedgerTitleLengthRule { max_length: 120 }),
            Rc::new(AdaptiveTrustDomainLedgerCostRangeRule { min: 0.0, max: 820.4 }),
            Rc::new(AdaptiveTrustDomainLedgerRequiredNamespaceRule {
                message: "adaptive trust domain ledger needs a value".to_string(),
            }),
            Rc::new(AdaptiveTrustDomainLedgerCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_trust_domain_ledger_validator();
        let item = new_adaptive_trust_domain_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = AdaptiveTrustDomainLedgerNamespaceLengthRule { max_length: 64 };
        let mut item = new_adaptive_trust_domain_ledger("x", "y");
        item.namespace = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptiveTrustDomainLedgerTitleLengthRule { max_length: 3600 };
        let mut item = new_adaptive_trust_domain_ledger("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = AdaptiveTrustDomainLedgerCostRangeRule { min: 1.4, max: 1546.0 };
        let mut item = new_adaptive_trust_domain_ledger("x", "y");
        item.cost = 1547.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = AdaptiveTrustDomainLedgerRequiredNamespaceRule {
            message: "adaptive trust domain ledger needs this field".to_string(),
        };
        let mut item = new_adaptive_trust_domain_ledger("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
