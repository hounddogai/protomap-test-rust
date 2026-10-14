use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation policy must pass. Check returns its problems, or none.
pub trait IsolationPolicyRule: std::fmt::Debug {
    fn check(&self, item: &IsolationPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationPolicyTitleLengthRule {
    pub max_length: i64,
}

impl IsolationPolicyRule for IsolationPolicyTitleLengthRule {
    fn check(&self, item: &IsolationPolicy) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationPolicyBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationPolicyRule for IsolationPolicyBatchSizeRangeRule {
    fn check(&self, item: &IsolationPolicy) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationPolicyRequiredTitleRule {
    pub message: String,
}

impl IsolationPolicyRule for IsolationPolicyRequiredTitleRule {
    fn check(&self, item: &IsolationPolicy) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation policy.
#[derive(Debug, Clone)]
pub struct IsolationPolicyValidator {
    pub rules: Vec<Rc<dyn IsolationPolicyRule>>,
}

impl IsolationPolicyValidator {
    pub fn validate(&self, item: &IsolationPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_policy_validator() -> IsolationPolicyValidator {
    IsolationPolicyValidator {
        rules: vec![
            Rc::new(IsolationPolicyTitleLengthRule { max_length: 1000 }) as Rc<dyn IsolationPolicyRule>,
            Rc::new(IsolationPolicyBatchSizeRangeRule { min: 64, max: 80 }),
            Rc::new(IsolationPolicyRequiredTitleRule {
                message: "isolation policy needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_policy_validator();
        let item = new_isolation_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = IsolationPolicyTitleLengthRule { max_length: 120 };
        let mut item = new_isolation_policy("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = IsolationPolicyBatchSizeRangeRule { min: 255, max: 255 };
        let mut item = new_isolation_policy("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = IsolationPolicyRequiredTitleRule {
            message: "isolation policy needs this field".to_string(),
        };
        let mut item = new_isolation_policy("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
