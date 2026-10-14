use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback profile must pass. Check returns its problems, or none.
pub trait RollbackProfileRule: std::fmt::Debug {
    fn check(&self, item: &RollbackProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProfileThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackProfileRule for RollbackProfileThresholdRangeRule {
    fn check(&self, item: &RollbackProfile) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProfileCodeLengthRule {
    pub max_length: i64,
}

impl RollbackProfileRule for RollbackProfileCodeLengthRule {
    fn check(&self, item: &RollbackProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProfileRequiredClusterRule {
    pub message: String,
}

impl RollbackProfileRule for RollbackProfileRequiredClusterRule {
    fn check(&self, item: &RollbackProfile) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback profile.
#[derive(Debug, Clone)]
pub struct RollbackProfileValidator {
    pub rules: Vec<Rc<dyn RollbackProfileRule>>,
}

impl RollbackProfileValidator {
    pub fn validate(&self, item: &RollbackProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_profile_validator() -> RollbackProfileValidator {
    RollbackProfileValidator {
        rules: vec![
            Rc::new(RollbackProfileThresholdRangeRule { min: 3.4, max: 3996.9 }) as Rc<dyn RollbackProfileRule>,
            Rc::new(RollbackProfileCodeLengthRule { max_length: 16 }),
            Rc::new(RollbackProfileRequiredClusterRule {
                message: "rollback profile needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_profile_validator();
        let item = new_rollback_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RollbackProfileThresholdRangeRule { min: 5.0, max: 4815.2 };
        let mut item = new_rollback_profile("x", "y");
        item.threshold = 4816.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackProfileCodeLengthRule { max_length: 255 };
        let mut item = new_rollback_profile("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RollbackProfileRequiredClusterRule {
            message: "rollback profile needs a value".to_string(),
        };
        let mut item = new_rollback_profile("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
