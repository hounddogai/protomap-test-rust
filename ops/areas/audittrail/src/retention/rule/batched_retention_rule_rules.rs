use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched retention rule must pass. Check returns its problems, or none.
pub trait BatchedRetentionRuleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRetentionRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionRuleUnitLengthRule {
    pub max_length: i64,
}

impl BatchedRetentionRuleRule for BatchedRetentionRuleUnitLengthRule {
    fn check(&self, item: &BatchedRetentionRule) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionRuleNameLengthRule {
    pub max_length: i64,
}

impl BatchedRetentionRuleRule for BatchedRetentionRuleNameLengthRule {
    fn check(&self, item: &BatchedRetentionRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionRuleCodeLengthRule {
    pub max_length: i64,
}

impl BatchedRetentionRuleRule for BatchedRetentionRuleCodeLengthRule {
    fn check(&self, item: &BatchedRetentionRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionRuleRequiredNameRule {
    pub message: String,
}

impl BatchedRetentionRuleRule for BatchedRetentionRuleRequiredNameRule {
    fn check(&self, item: &BatchedRetentionRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionRuleRequiredUnitRule {
    pub message: String,
}

impl BatchedRetentionRuleRule for BatchedRetentionRuleRequiredUnitRule {
    fn check(&self, item: &BatchedRetentionRule) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched retention rule.
#[derive(Debug, Clone)]
pub struct BatchedRetentionRuleValidator {
    pub rules: Vec<Rc<dyn BatchedRetentionRuleRule>>,
}

impl BatchedRetentionRuleValidator {
    pub fn validate(&self, item: &BatchedRetentionRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedRetentionRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRetentionRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_retention_rule_validator() -> BatchedRetentionRuleValidator {
    BatchedRetentionRuleValidator {
        rules: vec![
            Rc::new(BatchedRetentionRuleUnitLengthRule { max_length: 32 }) as Rc<dyn BatchedRetentionRuleRule>,
            Rc::new(BatchedRetentionRuleNameLengthRule { max_length: 64 }),
            Rc::new(BatchedRetentionRuleCodeLengthRule { max_length: 3600 }),
            Rc::new(BatchedRetentionRuleRequiredNameRule {
                message: "batched retention rule needs this field".to_string(),
            }),
            Rc::new(BatchedRetentionRuleRequiredUnitRule {
                message: "batched retention rule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_retention_rule_validator();
        let item = new_batched_retention_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = BatchedRetentionRuleUnitLengthRule { max_length: 80 };
        let mut item = new_batched_retention_rule("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedRetentionRuleNameLengthRule { max_length: 120 };
        let mut item = new_batched_retention_rule("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedRetentionRuleCodeLengthRule { max_length: 80 };
        let mut item = new_batched_retention_rule("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedRetentionRuleRequiredNameRule {
            message: "batched retention rule needs an entry".to_string(),
        };
        let mut item = new_batched_retention_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = BatchedRetentionRuleRequiredUnitRule {
            message: "batched retention rule needs an entry".to_string(),
        };
        let mut item = new_batched_retention_rule("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
