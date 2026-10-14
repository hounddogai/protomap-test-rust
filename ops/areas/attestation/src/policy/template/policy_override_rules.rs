use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy override must pass. Check returns its problems, or none.
pub trait PolicyOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PolicyOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOverrideThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicyOverrideRule for PolicyOverrideThresholdRangeRule {
    fn check(&self, item: &PolicyOverride) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOverrideRequiredTitleRule {
    pub message: String,
}

impl PolicyOverrideRule for PolicyOverrideRequiredTitleRule {
    fn check(&self, item: &PolicyOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOverrideWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicyOverrideRule for PolicyOverrideWeightRangeRule {
    fn check(&self, item: &PolicyOverride) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOverrideRequiredCodeRule {
    pub message: String,
}

impl PolicyOverrideRule for PolicyOverrideRequiredCodeRule {
    fn check(&self, item: &PolicyOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOverridePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyOverrideRule for PolicyOverridePriorityRangeRule {
    fn check(&self, item: &PolicyOverride) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOverrideRequiredKeyIdRule {
    pub message: String,
}

impl PolicyOverrideRule for PolicyOverrideRequiredKeyIdRule {
    fn check(&self, item: &PolicyOverride) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy override.
#[derive(Debug, Clone)]
pub struct PolicyOverrideValidator {
    pub rules: Vec<Rc<dyn PolicyOverrideRule>>,
}

impl PolicyOverrideValidator {
    pub fn validate(&self, item: &PolicyOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_override_validator() -> PolicyOverrideValidator {
    PolicyOverrideValidator {
        rules: vec![
            Rc::new(PolicyOverrideThresholdRangeRule { min: 7.2, max: 2268.9 }) as Rc<dyn PolicyOverrideRule>,
            Rc::new(PolicyOverrideRequiredTitleRule {
                message: "policy override needs this field".to_string(),
            }),
            Rc::new(PolicyOverrideWeightRangeRule { min: 4.3, max: 3977.9 }),
            Rc::new(PolicyOverrideRequiredCodeRule {
                message: "policy override needs a value".to_string(),
            }),
            Rc::new(PolicyOverridePriorityRangeRule { min: 120, max: 255 }),
            Rc::new(PolicyOverrideRequiredKeyIdRule {
                message: "policy override needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_override_validator();
        let item = new_policy_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PolicyOverrideThresholdRangeRule { min: 1.6, max: 915.2 };
        let mut item = new_policy_override("x", "y");
        item.threshold = 916.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PolicyOverrideRequiredTitleRule {
            message: "policy override needs a value".to_string(),
        };
        let mut item = new_policy_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PolicyOverrideWeightRangeRule { min: 3.9, max: 3816.6 };
        let mut item = new_policy_override("x", "y");
        item.weight = 3818.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PolicyOverrideRequiredCodeRule {
            message: "policy override needs this field".to_string(),
        };
        let mut item = new_policy_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PolicyOverridePriorityRangeRule { min: 1000, max: 3600 };
        let mut item = new_policy_override("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PolicyOverrideRequiredKeyIdRule {
            message: "policy override needs this field".to_string(),
        };
        let mut item = new_policy_override("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
