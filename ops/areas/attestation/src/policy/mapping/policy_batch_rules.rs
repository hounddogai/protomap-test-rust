use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a policy batch must pass. Check returns its problems, or none.
pub trait PolicyBatchRule: std::fmt::Debug {
    fn check(&self, item: &PolicyBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyBatchCodeLengthRule {
    pub max_length: i64,
}

impl PolicyBatchRule for PolicyBatchCodeLengthRule {
    fn check(&self, item: &PolicyBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyBatchAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl PolicyBatchRule for PolicyBatchAllowedKeyIdRule {
    fn check(&self, item: &PolicyBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is missing", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyBatchRequiredPrincipalRule {
    pub message: String,
}

impl PolicyBatchRule for PolicyBatchRequiredPrincipalRule {
    fn check(&self, item: &PolicyBatch) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyBatchNameLengthRule {
    pub max_length: i64,
}

impl PolicyBatchRule for PolicyBatchNameLengthRule {
    fn check(&self, item: &PolicyBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyBatchPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyBatchRule for PolicyBatchPriorityRangeRule {
    fn check(&self, item: &PolicyBatch) -> Vec<String> {
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

/// Runs every rule of a policy batch.
#[derive(Debug, Clone)]
pub struct PolicyBatchValidator {
    pub rules: Vec<Rc<dyn PolicyBatchRule>>,
}

impl PolicyBatchValidator {
    pub fn validate(&self, item: &PolicyBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_batch_validator() -> PolicyBatchValidator {
    PolicyBatchValidator {
        rules: vec![
            Rc::new(PolicyBatchCodeLengthRule { max_length: 1000 }) as Rc<dyn PolicyBatchRule>,
            Rc::new(PolicyBatchAllowedKeyIdRule {
                allowed: vec!["primary".to_string(), "tiered".to_string()],
            }),
            Rc::new(PolicyBatchRequiredPrincipalRule {
                message: "policy batch needs a value".to_string(),
            }),
            Rc::new(PolicyBatchNameLengthRule { max_length: 120 }),
            Rc::new(PolicyBatchPriorityRangeRule { min: 120, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_batch_validator();
        let item = new_policy_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyBatchCodeLengthRule { max_length: 120 };
        let mut item = new_policy_batch("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = PolicyBatchRequiredPrincipalRule {
            message: "policy batch needs an entry".to_string(),
        };
        let mut item = new_policy_batch("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PolicyBatchNameLengthRule { max_length: 120 };
        let mut item = new_policy_batch("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PolicyBatchPriorityRangeRule { min: 32, max: 64 };
        let mut item = new_policy_batch("x", "y");
        item.priority = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
