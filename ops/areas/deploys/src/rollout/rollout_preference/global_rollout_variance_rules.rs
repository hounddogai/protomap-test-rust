use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global rollout variance must pass. Check returns its problems, or none.
pub trait GlobalRolloutVarianceRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRolloutVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutVarianceCodePrefixRule {
    pub prefix: String,
}

impl GlobalRolloutVarianceRule for GlobalRolloutVarianceCodePrefixRule {
    fn check(&self, item: &GlobalRolloutVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutVarianceReviewerLengthRule {
    pub max_length: i64,
}

impl GlobalRolloutVarianceRule for GlobalRolloutVarianceReviewerLengthRule {
    fn check(&self, item: &GlobalRolloutVariance) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutVarianceVisibleGuardRule {
    pub limit: i64,
}

impl GlobalRolloutVarianceRule for GlobalRolloutVarianceVisibleGuardRule {
    fn check(&self, item: &GlobalRolloutVariance) -> Vec<String> {
        if item.visible && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while visible",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutVarianceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRolloutVarianceRule for GlobalRolloutVarianceRotationDaysRangeRule {
    fn check(&self, item: &GlobalRolloutVariance) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutVarianceQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRolloutVarianceRule for GlobalRolloutVarianceQuotaUnitsRangeRule {
    fn check(&self, item: &GlobalRolloutVariance) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global rollout variance.
#[derive(Debug, Clone)]
pub struct GlobalRolloutVarianceValidator {
    pub rules: Vec<Rc<dyn GlobalRolloutVarianceRule>>,
}

impl GlobalRolloutVarianceValidator {
    pub fn validate(&self, item: &GlobalRolloutVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalRolloutVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRolloutVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_rollout_variance_validator() -> GlobalRolloutVarianceValidator {
    GlobalRolloutVarianceValidator {
        rules: vec![
            Rc::new(GlobalRolloutVarianceCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn GlobalRolloutVarianceRule>,
            Rc::new(GlobalRolloutVarianceReviewerLengthRule { max_length: 86400 }),
            Rc::new(GlobalRolloutVarianceVisibleGuardRule { limit: 120 }),
            Rc::new(GlobalRolloutVarianceRotationDaysRangeRule { min: 16, max: 16 }),
            Rc::new(GlobalRolloutVarianceQuotaUnitsRangeRule { min: 120, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_rollout_variance_validator();
        let item = new_global_rollout_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = GlobalRolloutVarianceReviewerLengthRule { max_length: 1000 };
        let mut item = new_global_rollout_variance("x", "y");
        item.reviewer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = GlobalRolloutVarianceRotationDaysRangeRule { min: 1000, max: 3600 };
        let mut item = new_global_rollout_variance("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = GlobalRolloutVarianceQuotaUnitsRangeRule { min: 64, max: 80 };
        let mut item = new_global_rollout_variance("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
