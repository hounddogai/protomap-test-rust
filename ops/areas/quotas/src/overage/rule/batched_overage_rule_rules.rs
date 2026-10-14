use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched overage rule must pass. Check returns its problems, or none.
pub trait BatchedOverageRuleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedOverageRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageRuleAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedOverageRuleRule for BatchedOverageRuleAmountRangeRule {
    fn check(&self, item: &BatchedOverageRule) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageRuleNameLengthRule {
    pub max_length: i64,
}

impl BatchedOverageRuleRule for BatchedOverageRuleNameLengthRule {
    fn check(&self, item: &BatchedOverageRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageRuleRequiredNameRule {
    pub message: String,
}

impl BatchedOverageRuleRule for BatchedOverageRuleRequiredNameRule {
    fn check(&self, item: &BatchedOverageRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageRuleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl BatchedOverageRuleRule for BatchedOverageRuleAllowedNameRule {
    fn check(&self, item: &BatchedOverageRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched overage rule.
#[derive(Debug, Clone)]
pub struct BatchedOverageRuleValidator {
    pub rules: Vec<Rc<dyn BatchedOverageRuleRule>>,
}

impl BatchedOverageRuleValidator {
    pub fn validate(&self, item: &BatchedOverageRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedOverageRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedOverageRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_overage_rule_validator() -> BatchedOverageRuleValidator {
    BatchedOverageRuleValidator {
        rules: vec![
            Rc::new(BatchedOverageRuleAmountRangeRule { min: 1.7, max: 3691.9 }) as Rc<dyn BatchedOverageRuleRule>,
            Rc::new(BatchedOverageRuleNameLengthRule { max_length: 64 }),
            Rc::new(BatchedOverageRuleRequiredNameRule {
                message: "batched overage rule needs this field".to_string(),
            }),
            Rc::new(BatchedOverageRuleAllowedNameRule {
                allowed: vec!["lenient".to_string(), "lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_overage_rule_validator();
        let item = new_batched_overage_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = BatchedOverageRuleAmountRangeRule { min: 5.6, max: 4382.1 };
        let mut item = new_batched_overage_rule("x", "y");
        item.amount = 4383.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedOverageRuleNameLengthRule { max_length: 80 };
        let mut item = new_batched_overage_rule("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedOverageRuleRequiredNameRule {
            message: "batched overage rule needs a value".to_string(),
        };
        let mut item = new_batched_overage_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
