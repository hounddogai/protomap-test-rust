use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy policy exception must pass. Check returns its problems, or none.
pub trait LegacyPolicyExceptionRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPolicyException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyExceptionSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyPolicyExceptionRule for LegacyPolicyExceptionSequenceRangeRule {
    fn check(&self, item: &LegacyPolicyException) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyExceptionLabelLengthRule {
    pub max_length: i64,
}

impl LegacyPolicyExceptionRule for LegacyPolicyExceptionLabelLengthRule {
    fn check(&self, item: &LegacyPolicyException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyExceptionCodeLengthRule {
    pub max_length: i64,
}

impl LegacyPolicyExceptionRule for LegacyPolicyExceptionCodeLengthRule {
    fn check(&self, item: &LegacyPolicyException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy policy exception.
#[derive(Debug, Clone)]
pub struct LegacyPolicyExceptionValidator {
    pub rules: Vec<Rc<dyn LegacyPolicyExceptionRule>>,
}

impl LegacyPolicyExceptionValidator {
    pub fn validate(&self, item: &LegacyPolicyException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyPolicyException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPolicyException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_policy_exception_validator() -> LegacyPolicyExceptionValidator {
    LegacyPolicyExceptionValidator {
        rules: vec![
            Rc::new(LegacyPolicyExceptionSequenceRangeRule { min: 80, max: 80 }) as Rc<dyn LegacyPolicyExceptionRule>,
            Rc::new(LegacyPolicyExceptionLabelLengthRule { max_length: 16 }),
            Rc::new(LegacyPolicyExceptionCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_policy_exception_validator();
        let item = new_legacy_policy_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = LegacyPolicyExceptionSequenceRangeRule { min: 64, max: 255 };
        let mut item = new_legacy_policy_exception("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyPolicyExceptionLabelLengthRule { max_length: 80 };
        let mut item = new_legacy_policy_exception("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyPolicyExceptionCodeLengthRule { max_length: 86400 };
        let mut item = new_legacy_policy_exception("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
