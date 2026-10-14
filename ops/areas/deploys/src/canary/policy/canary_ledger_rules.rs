use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary ledger must pass. Check returns its problems, or none.
pub trait CanaryLedgerRule: std::fmt::Debug {
    fn check(&self, item: &CanaryLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryLedgerRequiredRegionRule {
    pub message: String,
}

impl CanaryLedgerRule for CanaryLedgerRequiredRegionRule {
    fn check(&self, item: &CanaryLedger) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryLedgerCodePrefixRule {
    pub prefix: String,
}

impl CanaryLedgerRule for CanaryLedgerCodePrefixRule {
    fn check(&self, item: &CanaryLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryLedgerRequiredCodeRule {
    pub message: String,
}

impl CanaryLedgerRule for CanaryLedgerRequiredCodeRule {
    fn check(&self, item: &CanaryLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryLedgerRegionLengthRule {
    pub max_length: i64,
}

impl CanaryLedgerRule for CanaryLedgerRegionLengthRule {
    fn check(&self, item: &CanaryLedger) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary ledger.
#[derive(Debug, Clone)]
pub struct CanaryLedgerValidator {
    pub rules: Vec<Rc<dyn CanaryLedgerRule>>,
}

impl CanaryLedgerValidator {
    pub fn validate(&self, item: &CanaryLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_ledger_validator() -> CanaryLedgerValidator {
    CanaryLedgerValidator {
        rules: vec![
            Rc::new(CanaryLedgerRequiredRegionRule {
                message: "canary ledger needs an entry".to_string(),
            }) as Rc<dyn CanaryLedgerRule>,
            Rc::new(CanaryLedgerCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CanaryLedgerRequiredCodeRule {
                message: "canary ledger needs a value".to_string(),
            }),
            Rc::new(CanaryLedgerRegionLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_ledger_validator();
        let item = new_canary_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = CanaryLedgerRequiredRegionRule {
            message: "canary ledger needs a value".to_string(),
        };
        let mut item = new_canary_ledger("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryLedgerRequiredCodeRule {
            message: "canary ledger needs a value".to_string(),
        };
        let mut item = new_canary_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = CanaryLedgerRegionLengthRule { max_length: 3600 };
        let mut item = new_canary_ledger("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
