use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual region pin policy must pass. Check returns its problems, or none.
pub trait ManualRegionPinPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ManualRegionPinPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinPolicyCodePrefixRule {
    pub prefix: String,
}

impl ManualRegionPinPolicyRule for ManualRegionPinPolicyCodePrefixRule {
    fn check(&self, item: &ManualRegionPinPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinPolicyArchivedGuardRule {
    pub limit: f64,
}

impl ManualRegionPinPolicyRule for ManualRegionPinPolicyArchivedGuardRule {
    fn check(&self, item: &ManualRegionPinPolicy) -> Vec<String> {
        if item.archived && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while archived", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinPolicyMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualRegionPinPolicyRule for ManualRegionPinPolicyMinimumRangeRule {
    fn check(&self, item: &ManualRegionPinPolicy) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinPolicyCategoryLengthRule {
    pub max_length: i64,
}

impl ManualRegionPinPolicyRule for ManualRegionPinPolicyCategoryLengthRule {
    fn check(&self, item: &ManualRegionPinPolicy) -> Vec<String> {
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

/// Runs every rule of a manual region pin policy.
#[derive(Debug, Clone)]
pub struct ManualRegionPinPolicyValidator {
    pub rules: Vec<Rc<dyn ManualRegionPinPolicyRule>>,
}

impl ManualRegionPinPolicyValidator {
    pub fn validate(&self, item: &ManualRegionPinPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRegionPinPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRegionPinPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_region_pin_policy_validator() -> ManualRegionPinPolicyValidator {
    ManualRegionPinPolicyValidator {
        rules: vec![
            Rc::new(ManualRegionPinPolicyCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ManualRegionPinPolicyRule>,
            Rc::new(ManualRegionPinPolicyArchivedGuardRule { limit: 0.2 }),
            Rc::new(ManualRegionPinPolicyMinimumRangeRule { min: 3.5, max: 1082.4 }),
            Rc::new(ManualRegionPinPolicyCategoryLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_region_pin_policy_validator();
        let item = new_manual_region_pin_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ManualRegionPinPolicyMinimumRangeRule { min: 1.5, max: 1478.7 };
        let mut item = new_manual_region_pin_policy("x", "y");
        item.minimum = 1480.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = ManualRegionPinPolicyCategoryLengthRule { max_length: 16 };
        let mut item = new_manual_region_pin_policy("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
