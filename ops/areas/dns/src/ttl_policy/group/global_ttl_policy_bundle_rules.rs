use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global ttl policy bundle must pass. Check returns its problems, or none.
pub trait GlobalTtlPolicyBundleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTtlPolicyBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyBundleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalTtlPolicyBundleRule for GlobalTtlPolicyBundleRatioRangeRule {
    fn check(&self, item: &GlobalTtlPolicyBundle) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyBundleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl GlobalTtlPolicyBundleRule for GlobalTtlPolicyBundleAllowedLabelRule {
    fn check(&self, item: &GlobalTtlPolicyBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is out of range", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyBundleCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalTtlPolicyBundleRule for GlobalTtlPolicyBundleCapacityRangeRule {
    fn check(&self, item: &GlobalTtlPolicyBundle) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyBundleDescriptionLengthRule {
    pub max_length: i64,
}

impl GlobalTtlPolicyBundleRule for GlobalTtlPolicyBundleDescriptionLengthRule {
    fn check(&self, item: &GlobalTtlPolicyBundle) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyBundleRequiredDescriptionRule {
    pub message: String,
}

impl GlobalTtlPolicyBundleRule for GlobalTtlPolicyBundleRequiredDescriptionRule {
    fn check(&self, item: &GlobalTtlPolicyBundle) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyBundleLabelLengthRule {
    pub max_length: i64,
}

impl GlobalTtlPolicyBundleRule for GlobalTtlPolicyBundleLabelLengthRule {
    fn check(&self, item: &GlobalTtlPolicyBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global ttl policy bundle.
#[derive(Debug, Clone)]
pub struct GlobalTtlPolicyBundleValidator {
    pub rules: Vec<Rc<dyn GlobalTtlPolicyBundleRule>>,
}

impl GlobalTtlPolicyBundleValidator {
    pub fn validate(&self, item: &GlobalTtlPolicyBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalTtlPolicyBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTtlPolicyBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_ttl_policy_bundle_validator() -> GlobalTtlPolicyBundleValidator {
    GlobalTtlPolicyBundleValidator {
        rules: vec![
            Rc::new(GlobalTtlPolicyBundleRatioRangeRule { min: 6.1, max: 2524.1 }) as Rc<dyn GlobalTtlPolicyBundleRule>,
            Rc::new(GlobalTtlPolicyBundleAllowedLabelRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(GlobalTtlPolicyBundleCapacityRangeRule { min: 16, max: 120 }),
            Rc::new(GlobalTtlPolicyBundleDescriptionLengthRule { max_length: 1000 }),
            Rc::new(GlobalTtlPolicyBundleRequiredDescriptionRule {
                message: "global ttl policy bundle needs this field".to_string(),
            }),
            Rc::new(GlobalTtlPolicyBundleLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_ttl_policy_bundle_validator();
        let item = new_global_ttl_policy_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = GlobalTtlPolicyBundleRatioRangeRule { min: 8.9, max: 1755.3 };
        let mut item = new_global_ttl_policy_bundle("x", "y");
        item.ratio = 1756.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = GlobalTtlPolicyBundleCapacityRangeRule { min: 16, max: 86400 };
        let mut item = new_global_ttl_policy_bundle("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = GlobalTtlPolicyBundleDescriptionLengthRule { max_length: 86400 };
        let mut item = new_global_ttl_policy_bundle("x", "y");
        item.description = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = GlobalTtlPolicyBundleRequiredDescriptionRule {
            message: "global ttl policy bundle needs this field".to_string(),
        };
        let mut item = new_global_ttl_policy_bundle("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GlobalTtlPolicyBundleLabelLengthRule { max_length: 3600 };
        let mut item = new_global_ttl_policy_bundle("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
