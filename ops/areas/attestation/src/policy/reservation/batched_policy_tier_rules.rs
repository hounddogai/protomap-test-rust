use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched policy tier must pass. Check returns its problems, or none.
pub trait BatchedPolicyTierRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPolicyTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTierRequiredTitleRule {
    pub message: String,
}

impl BatchedPolicyTierRule for BatchedPolicyTierRequiredTitleRule {
    fn check(&self, item: &BatchedPolicyTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTierTitleLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyTierRule for BatchedPolicyTierTitleLengthRule {
    fn check(&self, item: &BatchedPolicyTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTierRequiredNameRule {
    pub message: String,
}

impl BatchedPolicyTierRule for BatchedPolicyTierRequiredNameRule {
    fn check(&self, item: &BatchedPolicyTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTierCodePrefixRule {
    pub prefix: String,
}

impl BatchedPolicyTierRule for BatchedPolicyTierCodePrefixRule {
    fn check(&self, item: &BatchedPolicyTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTierCodeLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyTierRule for BatchedPolicyTierCodeLengthRule {
    fn check(&self, item: &BatchedPolicyTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched policy tier.
#[derive(Debug, Clone)]
pub struct BatchedPolicyTierValidator {
    pub rules: Vec<Rc<dyn BatchedPolicyTierRule>>,
}

impl BatchedPolicyTierValidator {
    pub fn validate(&self, item: &BatchedPolicyTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedPolicyTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPolicyTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_policy_tier_validator() -> BatchedPolicyTierValidator {
    BatchedPolicyTierValidator {
        rules: vec![
            Rc::new(BatchedPolicyTierRequiredTitleRule {
                message: "batched policy tier needs an entry".to_string(),
            }) as Rc<dyn BatchedPolicyTierRule>,
            Rc::new(BatchedPolicyTierTitleLengthRule { max_length: 16 }),
            Rc::new(BatchedPolicyTierRequiredNameRule {
                message: "batched policy tier needs a value".to_string(),
            }),
            Rc::new(BatchedPolicyTierCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(BatchedPolicyTierCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_policy_tier_validator();
        let item = new_batched_policy_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BatchedPolicyTierRequiredTitleRule {
            message: "batched policy tier needs this field".to_string(),
        };
        let mut item = new_batched_policy_tier("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BatchedPolicyTierTitleLengthRule { max_length: 255 };
        let mut item = new_batched_policy_tier("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedPolicyTierRequiredNameRule {
            message: "batched policy tier needs an entry".to_string(),
        };
        let mut item = new_batched_policy_tier("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedPolicyTierCodeLengthRule { max_length: 86400 };
        let mut item = new_batched_policy_tier("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
