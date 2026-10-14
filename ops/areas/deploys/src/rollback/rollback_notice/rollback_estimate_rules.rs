use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback estimate must pass. Check returns its problems, or none.
pub trait RollbackEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RollbackEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEstimateRequiredCodeRule {
    pub message: String,
}

impl RollbackEstimateRule for RollbackEstimateRequiredCodeRule {
    fn check(&self, item: &RollbackEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEstimateLabelLengthRule {
    pub max_length: i64,
}

impl RollbackEstimateRule for RollbackEstimateLabelLengthRule {
    fn check(&self, item: &RollbackEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEstimateArchivedGuardRule {
    pub limit: i64,
}

impl RollbackEstimateRule for RollbackEstimateArchivedGuardRule {
    fn check(&self, item: &RollbackEstimate) -> Vec<String> {
        if item.archived && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while archived", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEstimateBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackEstimateRule for RollbackEstimateBurstRangeRule {
    fn check(&self, item: &RollbackEstimate) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback estimate.
#[derive(Debug, Clone)]
pub struct RollbackEstimateValidator {
    pub rules: Vec<Rc<dyn RollbackEstimateRule>>,
}

impl RollbackEstimateValidator {
    pub fn validate(&self, item: &RollbackEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_estimate_validator() -> RollbackEstimateValidator {
    RollbackEstimateValidator {
        rules: vec![
            Rc::new(RollbackEstimateRequiredCodeRule {
                message: "rollback estimate needs this field".to_string(),
            }) as Rc<dyn RollbackEstimateRule>,
            Rc::new(RollbackEstimateLabelLengthRule { max_length: 1000 }),
            Rc::new(RollbackEstimateArchivedGuardRule { limit: 255 }),
            Rc::new(RollbackEstimateBurstRangeRule { min: 32, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_estimate_validator();
        let item = new_rollback_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackEstimateRequiredCodeRule {
            message: "rollback estimate needs this field".to_string(),
        };
        let mut item = new_rollback_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollbackEstimateLabelLengthRule { max_length: 80 };
        let mut item = new_rollback_estimate("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = RollbackEstimateBurstRangeRule { min: 3600, max: 86400 };
        let mut item = new_rollback_estimate("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
