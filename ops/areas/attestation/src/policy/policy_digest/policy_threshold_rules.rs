use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy threshold must pass. Check returns its problems, or none.
pub trait PolicyThresholdRule: std::fmt::Debug {
    fn check(&self, item: &PolicyThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyThresholdMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicyThresholdRule for PolicyThresholdMinimumRangeRule {
    fn check(&self, item: &PolicyThreshold) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyThresholdCodeLengthRule {
    pub max_length: i64,
}

impl PolicyThresholdRule for PolicyThresholdCodeLengthRule {
    fn check(&self, item: &PolicyThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyThresholdRequiredLabelRule {
    pub message: String,
}

impl PolicyThresholdRule for PolicyThresholdRequiredLabelRule {
    fn check(&self, item: &PolicyThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy threshold.
#[derive(Debug, Clone)]
pub struct PolicyThresholdValidator {
    pub rules: Vec<Rc<dyn PolicyThresholdRule>>,
}

impl PolicyThresholdValidator {
    pub fn validate(&self, item: &PolicyThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_threshold_validator() -> PolicyThresholdValidator {
    PolicyThresholdValidator {
        rules: vec![
            Rc::new(PolicyThresholdMinimumRangeRule { min: 5.4, max: 3614.7 }) as Rc<dyn PolicyThresholdRule>,
            Rc::new(PolicyThresholdCodeLengthRule { max_length: 16 }),
            Rc::new(PolicyThresholdRequiredLabelRule {
                message: "policy threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_threshold_validator();
        let item = new_policy_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PolicyThresholdMinimumRangeRule { min: 8.9, max: 723.4 };
        let mut item = new_policy_threshold("x", "y");
        item.minimum = 724.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyThresholdCodeLengthRule { max_length: 64 };
        let mut item = new_policy_threshold("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PolicyThresholdRequiredLabelRule {
            message: "policy threshold needs this field".to_string(),
        };
        let mut item = new_policy_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
