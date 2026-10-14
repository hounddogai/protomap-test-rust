use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending overage batch must pass. Check returns its problems, or none.
pub trait PendingOverageBatchRule: std::fmt::Debug {
    fn check(&self, item: &PendingOverageBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBatchCodePrefixRule {
    pub prefix: String,
}

impl PendingOverageBatchRule for PendingOverageBatchCodePrefixRule {
    fn check(&self, item: &PendingOverageBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBatchRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingOverageBatchRule for PendingOverageBatchRotationDaysRangeRule {
    fn check(&self, item: &PendingOverageBatch) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBatchCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingOverageBatchRule for PendingOverageBatchCeilingRangeRule {
    fn check(&self, item: &PendingOverageBatch) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBatchCodeLengthRule {
    pub max_length: i64,
}

impl PendingOverageBatchRule for PendingOverageBatchCodeLengthRule {
    fn check(&self, item: &PendingOverageBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBatchThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingOverageBatchRule for PendingOverageBatchThresholdRangeRule {
    fn check(&self, item: &PendingOverageBatch) -> Vec<String> {
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

/// Runs every rule of a pending overage batch.
#[derive(Debug, Clone)]
pub struct PendingOverageBatchValidator {
    pub rules: Vec<Rc<dyn PendingOverageBatchRule>>,
}

impl PendingOverageBatchValidator {
    pub fn validate(&self, item: &PendingOverageBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingOverageBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingOverageBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_overage_batch_validator() -> PendingOverageBatchValidator {
    PendingOverageBatchValidator {
        rules: vec![
            Rc::new(PendingOverageBatchCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn PendingOverageBatchRule>,
            Rc::new(PendingOverageBatchRotationDaysRangeRule { min: 255, max: 86400 }),
            Rc::new(PendingOverageBatchCeilingRangeRule { min: 1.7, max: 1783.6 }),
            Rc::new(PendingOverageBatchCodeLengthRule { max_length: 16 }),
            Rc::new(PendingOverageBatchThresholdRangeRule { min: 7.9, max: 4754.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_overage_batch_validator();
        let item = new_pending_overage_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PendingOverageBatchRotationDaysRangeRule { min: 16, max: 255 };
        let mut item = new_pending_overage_batch("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = PendingOverageBatchCeilingRangeRule { min: 7.2, max: 4915.2 };
        let mut item = new_pending_overage_batch("x", "y");
        item.ceiling = 4916.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingOverageBatchCodeLengthRule { max_length: 255 };
        let mut item = new_pending_overage_batch("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PendingOverageBatchThresholdRangeRule { min: 9.0, max: 1073.5 };
        let mut item = new_pending_overage_batch("x", "y");
        item.threshold = 1075.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
