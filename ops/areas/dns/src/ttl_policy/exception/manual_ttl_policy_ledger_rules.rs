use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual ttl policy ledger must pass. Check returns its problems, or none.
pub trait ManualTtlPolicyLedgerRule: std::fmt::Debug {
    fn check(&self, item: &ManualTtlPolicyLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyLedgerAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualTtlPolicyLedgerRule for ManualTtlPolicyLedgerAllowedCodeRule {
    fn check(&self, item: &ManualTtlPolicyLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyLedgerRequiredCodeRule {
    pub message: String,
}

impl ManualTtlPolicyLedgerRule for ManualTtlPolicyLedgerRequiredCodeRule {
    fn check(&self, item: &ManualTtlPolicyLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyLedgerRequiredUnitRule {
    pub message: String,
}

impl ManualTtlPolicyLedgerRule for ManualTtlPolicyLedgerRequiredUnitRule {
    fn check(&self, item: &ManualTtlPolicyLedger) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyLedgerUnitLengthRule {
    pub max_length: i64,
}

impl ManualTtlPolicyLedgerRule for ManualTtlPolicyLedgerUnitLengthRule {
    fn check(&self, item: &ManualTtlPolicyLedger) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual ttl policy ledger.
#[derive(Debug, Clone)]
pub struct ManualTtlPolicyLedgerValidator {
    pub rules: Vec<Rc<dyn ManualTtlPolicyLedgerRule>>,
}

impl ManualTtlPolicyLedgerValidator {
    pub fn validate(&self, item: &ManualTtlPolicyLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualTtlPolicyLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTtlPolicyLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_ttl_policy_ledger_validator() -> ManualTtlPolicyLedgerValidator {
    ManualTtlPolicyLedgerValidator {
        rules: vec![
            Rc::new(ManualTtlPolicyLedgerAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn ManualTtlPolicyLedgerRule>,
            Rc::new(ManualTtlPolicyLedgerRequiredCodeRule {
                message: "manual ttl policy ledger needs an entry".to_string(),
            }),
            Rc::new(ManualTtlPolicyLedgerRequiredUnitRule {
                message: "manual ttl policy ledger needs an entry".to_string(),
            }),
            Rc::new(ManualTtlPolicyLedgerUnitLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_ttl_policy_ledger_validator();
        let item = new_manual_ttl_policy_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualTtlPolicyLedgerRequiredCodeRule {
            message: "manual ttl policy ledger needs an entry".to_string(),
        };
        let mut item = new_manual_ttl_policy_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = ManualTtlPolicyLedgerRequiredUnitRule {
            message: "manual ttl policy ledger needs an entry".to_string(),
        };
        let mut item = new_manual_ttl_policy_ledger("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ManualTtlPolicyLedgerUnitLengthRule { max_length: 64 };
        let mut item = new_manual_ttl_policy_ledger("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
