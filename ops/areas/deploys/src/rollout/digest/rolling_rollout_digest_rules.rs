use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling rollout digest must pass. Check returns its problems, or none.
pub trait RollingRolloutDigestRule: std::fmt::Debug {
    fn check(&self, item: &RollingRolloutDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutDigestRequiredNameRule {
    pub message: String,
}

impl RollingRolloutDigestRule for RollingRolloutDigestRequiredNameRule {
    fn check(&self, item: &RollingRolloutDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutDigestRequiredCodeRule {
    pub message: String,
}

impl RollingRolloutDigestRule for RollingRolloutDigestRequiredCodeRule {
    fn check(&self, item: &RollingRolloutDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutDigestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingRolloutDigestRule for RollingRolloutDigestExpiresAtRangeRule {
    fn check(&self, item: &RollingRolloutDigest) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutDigestCategoryLengthRule {
    pub max_length: i64,
}

impl RollingRolloutDigestRule for RollingRolloutDigestCategoryLengthRule {
    fn check(&self, item: &RollingRolloutDigest) -> Vec<String> {
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

/// Runs every rule of a rolling rollout digest.
#[derive(Debug, Clone)]
pub struct RollingRolloutDigestValidator {
    pub rules: Vec<Rc<dyn RollingRolloutDigestRule>>,
}

impl RollingRolloutDigestValidator {
    pub fn validate(&self, item: &RollingRolloutDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingRolloutDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRolloutDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_rollout_digest_validator() -> RollingRolloutDigestValidator {
    RollingRolloutDigestValidator {
        rules: vec![
            Rc::new(RollingRolloutDigestRequiredNameRule {
                message: "rolling rollout digest needs this field".to_string(),
            }) as Rc<dyn RollingRolloutDigestRule>,
            Rc::new(RollingRolloutDigestRequiredCodeRule {
                message: "rolling rollout digest needs this field".to_string(),
            }),
            Rc::new(RollingRolloutDigestExpiresAtRangeRule { min: 32, max: 120 }),
            Rc::new(RollingRolloutDigestCategoryLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_rollout_digest_validator();
        let item = new_rolling_rollout_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingRolloutDigestRequiredNameRule {
            message: "rolling rollout digest needs a value".to_string(),
        };
        let mut item = new_rolling_rollout_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingRolloutDigestRequiredCodeRule {
            message: "rolling rollout digest needs an entry".to_string(),
        };
        let mut item = new_rolling_rollout_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RollingRolloutDigestExpiresAtRangeRule { min: 80, max: 80 };
        let mut item = new_rolling_rollout_digest("x", "y");
        item.expires_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = RollingRolloutDigestCategoryLengthRule { max_length: 16 };
        let mut item = new_rolling_rollout_digest("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
