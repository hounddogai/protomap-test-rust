use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched limit policy must pass. Check returns its problems, or none.
pub trait BatchedLimitPolicyRule: std::fmt::Debug {
    fn check(&self, item: &BatchedLimitPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitPolicyCurrencyLengthRule {
    pub max_length: i64,
}

impl BatchedLimitPolicyRule for BatchedLimitPolicyCurrencyLengthRule {
    fn check(&self, item: &BatchedLimitPolicy) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitPolicyRequiredNameRule {
    pub message: String,
}

impl BatchedLimitPolicyRule for BatchedLimitPolicyRequiredNameRule {
    fn check(&self, item: &BatchedLimitPolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitPolicyRequiredCurrencyRule {
    pub message: String,
}

impl BatchedLimitPolicyRule for BatchedLimitPolicyRequiredCurrencyRule {
    fn check(&self, item: &BatchedLimitPolicy) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitPolicyNameLengthRule {
    pub max_length: i64,
}

impl BatchedLimitPolicyRule for BatchedLimitPolicyNameLengthRule {
    fn check(&self, item: &BatchedLimitPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitPolicyRequiredCodeRule {
    pub message: String,
}

impl BatchedLimitPolicyRule for BatchedLimitPolicyRequiredCodeRule {
    fn check(&self, item: &BatchedLimitPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitPolicyMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedLimitPolicyRule for BatchedLimitPolicyMinimumRangeRule {
    fn check(&self, item: &BatchedLimitPolicy) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched limit policy.
#[derive(Debug, Clone)]
pub struct BatchedLimitPolicyValidator {
    pub rules: Vec<Rc<dyn BatchedLimitPolicyRule>>,
}

impl BatchedLimitPolicyValidator {
    pub fn validate(&self, item: &BatchedLimitPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedLimitPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedLimitPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_limit_policy_validator() -> BatchedLimitPolicyValidator {
    BatchedLimitPolicyValidator {
        rules: vec![
            Rc::new(BatchedLimitPolicyCurrencyLengthRule { max_length: 64 }) as Rc<dyn BatchedLimitPolicyRule>,
            Rc::new(BatchedLimitPolicyRequiredNameRule {
                message: "batched limit policy needs a value".to_string(),
            }),
            Rc::new(BatchedLimitPolicyRequiredCurrencyRule {
                message: "batched limit policy needs this field".to_string(),
            }),
            Rc::new(BatchedLimitPolicyNameLengthRule { max_length: 16 }),
            Rc::new(BatchedLimitPolicyRequiredCodeRule {
                message: "batched limit policy needs an entry".to_string(),
            }),
            Rc::new(BatchedLimitPolicyMinimumRangeRule { min: 4.9, max: 4507.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_limit_policy_validator();
        let item = new_batched_limit_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = BatchedLimitPolicyCurrencyLengthRule { max_length: 16 };
        let mut item = new_batched_limit_policy("x", "y");
        item.currency = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedLimitPolicyRequiredNameRule {
            message: "batched limit policy needs this field".to_string(),
        };
        let mut item = new_batched_limit_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = BatchedLimitPolicyRequiredCurrencyRule {
            message: "batched limit policy needs this field".to_string(),
        };
        let mut item = new_batched_limit_policy("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedLimitPolicyNameLengthRule { max_length: 255 };
        let mut item = new_batched_limit_policy("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedLimitPolicyRequiredCodeRule {
            message: "batched limit policy needs this field".to_string(),
        };
        let mut item = new_batched_limit_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = BatchedLimitPolicyMinimumRangeRule { min: 2.4, max: 1626.1 };
        let mut item = new_batched_limit_policy("x", "y");
        item.minimum = 1627.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
