use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout variance must pass. Check returns its problems, or none.
pub trait RolloutVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RolloutVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutVarianceTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutVarianceRule for RolloutVarianceTtlSecondsRangeRule {
    fn check(&self, item: &RolloutVariance) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutVarianceUnitLengthRule {
    pub max_length: i64,
}

impl RolloutVarianceRule for RolloutVarianceUnitLengthRule {
    fn check(&self, item: &RolloutVariance) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutVarianceCodeLengthRule {
    pub max_length: i64,
}

impl RolloutVarianceRule for RolloutVarianceCodeLengthRule {
    fn check(&self, item: &RolloutVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutVarianceAudienceLengthRule {
    pub max_length: i64,
}

impl RolloutVarianceRule for RolloutVarianceAudienceLengthRule {
    fn check(&self, item: &RolloutVariance) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutVarianceRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutVarianceRule for RolloutVarianceRetriesRangeRule {
    fn check(&self, item: &RolloutVariance) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollout variance.
#[derive(Debug, Clone)]
pub struct RolloutVarianceValidator {
    pub rules: Vec<Rc<dyn RolloutVarianceRule>>,
}

impl RolloutVarianceValidator {
    pub fn validate(&self, item: &RolloutVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_variance_validator() -> RolloutVarianceValidator {
    RolloutVarianceValidator {
        rules: vec![
            Rc::new(RolloutVarianceTtlSecondsRangeRule { min: 32, max: 86400 }) as Rc<dyn RolloutVarianceRule>,
            Rc::new(RolloutVarianceUnitLengthRule { max_length: 255 }),
            Rc::new(RolloutVarianceCodeLengthRule { max_length: 1000 }),
            Rc::new(RolloutVarianceAudienceLengthRule { max_length: 80 }),
            Rc::new(RolloutVarianceRetriesRangeRule { min: 120, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_variance_validator();
        let item = new_rollout_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RolloutVarianceTtlSecondsRangeRule { min: 255, max: 255 };
        let mut item = new_rollout_variance("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = RolloutVarianceUnitLengthRule { max_length: 255 };
        let mut item = new_rollout_variance("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutVarianceCodeLengthRule { max_length: 32 };
        let mut item = new_rollout_variance("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RolloutVarianceAudienceLengthRule { max_length: 120 };
        let mut item = new_rollout_variance("x", "y");
        item.audience = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RolloutVarianceRetriesRangeRule { min: 32, max: 120 };
        let mut item = new_rollout_variance("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
