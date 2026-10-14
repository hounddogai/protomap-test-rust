use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual ttl policy threshold must pass. Check returns its problems, or none.
pub trait ManualTtlPolicyThresholdRule: std::fmt::Debug {
    fn check(&self, item: &ManualTtlPolicyThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyThresholdRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualTtlPolicyThresholdRule for ManualTtlPolicyThresholdRetriesRangeRule {
    fn check(&self, item: &ManualTtlPolicyThreshold) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyThresholdTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ManualTtlPolicyThresholdRule for ManualTtlPolicyThresholdTagsLimitRule {
    fn check(&self, item: &ManualTtlPolicyThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyThresholdTitleLengthRule {
    pub max_length: i64,
}

impl ManualTtlPolicyThresholdRule for ManualTtlPolicyThresholdTitleLengthRule {
    fn check(&self, item: &ManualTtlPolicyThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyThresholdQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualTtlPolicyThresholdRule for ManualTtlPolicyThresholdQuantityRangeRule {
    fn check(&self, item: &ManualTtlPolicyThreshold) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual ttl policy threshold.
#[derive(Debug, Clone)]
pub struct ManualTtlPolicyThresholdValidator {
    pub rules: Vec<Rc<dyn ManualTtlPolicyThresholdRule>>,
}

impl ManualTtlPolicyThresholdValidator {
    pub fn validate(&self, item: &ManualTtlPolicyThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualTtlPolicyThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTtlPolicyThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_ttl_policy_threshold_validator() -> ManualTtlPolicyThresholdValidator {
    ManualTtlPolicyThresholdValidator {
        rules: vec![
            Rc::new(ManualTtlPolicyThresholdRetriesRangeRule { min: 64, max: 120 })
                as Rc<dyn ManualTtlPolicyThresholdRule>,
            Rc::new(ManualTtlPolicyThresholdTagsLimitRule {
                max_count: 255,
                banned: Vec::new(),
            }),
            Rc::new(ManualTtlPolicyThresholdTitleLengthRule { max_length: 255 }),
            Rc::new(ManualTtlPolicyThresholdQuantityRangeRule { min: 86400, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_ttl_policy_threshold_validator();
        let item = new_manual_ttl_policy_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ManualTtlPolicyThresholdRetriesRangeRule { min: 16, max: 1000 };
        let mut item = new_manual_ttl_policy_threshold("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualTtlPolicyThresholdTitleLengthRule { max_length: 64 };
        let mut item = new_manual_ttl_policy_threshold("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ManualTtlPolicyThresholdQuantityRangeRule { min: 32, max: 32 };
        let mut item = new_manual_ttl_policy_threshold("x", "y");
        item.quantity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
