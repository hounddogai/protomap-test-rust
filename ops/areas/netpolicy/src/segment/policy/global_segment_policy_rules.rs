use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global segment policy must pass. Check returns its problems, or none.
pub trait GlobalSegmentPolicyRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSegmentPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPolicyCategoryLengthRule {
    pub max_length: i64,
}

impl GlobalSegmentPolicyRule for GlobalSegmentPolicyCategoryLengthRule {
    fn check(&self, item: &GlobalSegmentPolicy) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPolicyCodeLengthRule {
    pub max_length: i64,
}

impl GlobalSegmentPolicyRule for GlobalSegmentPolicyCodeLengthRule {
    fn check(&self, item: &GlobalSegmentPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPolicyRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalSegmentPolicyRule for GlobalSegmentPolicyRetriesRangeRule {
    fn check(&self, item: &GlobalSegmentPolicy) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPolicyExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalSegmentPolicyRule for GlobalSegmentPolicyExpiresAtRangeRule {
    fn check(&self, item: &GlobalSegmentPolicy) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global segment policy.
#[derive(Debug, Clone)]
pub struct GlobalSegmentPolicyValidator {
    pub rules: Vec<Rc<dyn GlobalSegmentPolicyRule>>,
}

impl GlobalSegmentPolicyValidator {
    pub fn validate(&self, item: &GlobalSegmentPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalSegmentPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSegmentPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_segment_policy_validator() -> GlobalSegmentPolicyValidator {
    GlobalSegmentPolicyValidator {
        rules: vec![
            Rc::new(GlobalSegmentPolicyCategoryLengthRule { max_length: 64 }) as Rc<dyn GlobalSegmentPolicyRule>,
            Rc::new(GlobalSegmentPolicyCodeLengthRule { max_length: 86400 }),
            Rc::new(GlobalSegmentPolicyRetriesRangeRule { min: 16, max: 120 }),
            Rc::new(GlobalSegmentPolicyExpiresAtRangeRule { min: 80, max: 80 }),
        ],
    }
}
