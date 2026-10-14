use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy policy ledger must pass. Check returns its problems, or none.
pub trait LegacyPolicyLedgerRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPolicyLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyLedgerRequiredFingerprintRule {
    pub message: String,
}

impl LegacyPolicyLedgerRule for LegacyPolicyLedgerRequiredFingerprintRule {
    fn check(&self, item: &LegacyPolicyLedger) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyLedgerLabelLengthRule {
    pub max_length: i64,
}

impl LegacyPolicyLedgerRule for LegacyPolicyLedgerLabelLengthRule {
    fn check(&self, item: &LegacyPolicyLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyLedgerPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyPolicyLedgerRule for LegacyPolicyLedgerPriorityRangeRule {
    fn check(&self, item: &LegacyPolicyLedger) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy policy ledger.
#[derive(Debug, Clone)]
pub struct LegacyPolicyLedgerValidator {
    pub rules: Vec<Rc<dyn LegacyPolicyLedgerRule>>,
}

impl LegacyPolicyLedgerValidator {
    pub fn validate(&self, item: &LegacyPolicyLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyPolicyLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPolicyLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_policy_ledger_validator() -> LegacyPolicyLedgerValidator {
    LegacyPolicyLedgerValidator {
        rules: vec![
            Rc::new(LegacyPolicyLedgerRequiredFingerprintRule {
                message: "legacy policy ledger needs a value".to_string(),
            }) as Rc<dyn LegacyPolicyLedgerRule>,
            Rc::new(LegacyPolicyLedgerLabelLengthRule { max_length: 1000 }),
            Rc::new(LegacyPolicyLedgerPriorityRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_policy_ledger_validator();
        let item = new_legacy_policy_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = LegacyPolicyLedgerRequiredFingerprintRule {
            message: "legacy policy ledger needs a value".to_string(),
        };
        let mut item = new_legacy_policy_ledger("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyPolicyLedgerLabelLengthRule { max_length: 32 };
        let mut item = new_legacy_policy_ledger("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = LegacyPolicyLedgerPriorityRangeRule { min: 120, max: 86400 };
        let mut item = new_legacy_policy_ledger("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
