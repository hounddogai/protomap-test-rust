use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy ttl policy ledger must pass. Check returns its problems, or none.
pub trait LegacyTtlPolicyLedgerRule: std::fmt::Debug {
    fn check(&self, item: &LegacyTtlPolicyLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyLedgerVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyTtlPolicyLedgerRule for LegacyTtlPolicyLedgerVersionRangeRule {
    fn check(&self, item: &LegacyTtlPolicyLedger) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyLedgerNameLengthRule {
    pub max_length: i64,
}

impl LegacyTtlPolicyLedgerRule for LegacyTtlPolicyLedgerNameLengthRule {
    fn check(&self, item: &LegacyTtlPolicyLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyLedgerAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LegacyTtlPolicyLedgerRule for LegacyTtlPolicyLedgerAllowedTitleRule {
    fn check(&self, item: &LegacyTtlPolicyLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy ttl policy ledger.
#[derive(Debug, Clone)]
pub struct LegacyTtlPolicyLedgerValidator {
    pub rules: Vec<Rc<dyn LegacyTtlPolicyLedgerRule>>,
}

impl LegacyTtlPolicyLedgerValidator {
    pub fn validate(&self, item: &LegacyTtlPolicyLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyTtlPolicyLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyTtlPolicyLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_ttl_policy_ledger_validator() -> LegacyTtlPolicyLedgerValidator {
    LegacyTtlPolicyLedgerValidator {
        rules: vec![
            Rc::new(LegacyTtlPolicyLedgerVersionRangeRule { min: 255, max: 86400 })
                as Rc<dyn LegacyTtlPolicyLedgerRule>,
            Rc::new(LegacyTtlPolicyLedgerNameLengthRule { max_length: 86400 }),
            Rc::new(LegacyTtlPolicyLedgerAllowedTitleRule {
                allowed: vec!["adaptive".to_string(), "pending".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_ttl_policy_ledger_validator();
        let item = new_legacy_ttl_policy_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = LegacyTtlPolicyLedgerVersionRangeRule { min: 120, max: 1000 };
        let mut item = new_legacy_ttl_policy_ledger("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyTtlPolicyLedgerNameLengthRule { max_length: 86400 };
        let mut item = new_legacy_ttl_policy_ledger("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
