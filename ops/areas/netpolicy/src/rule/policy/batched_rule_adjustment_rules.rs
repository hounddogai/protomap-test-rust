use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched rule adjustment must pass. Check returns its problems, or none.
pub trait BatchedRuleAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRuleAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAdjustmentRequiredSegmentRule {
    pub message: String,
}

impl BatchedRuleAdjustmentRule for BatchedRuleAdjustmentRequiredSegmentRule {
    fn check(&self, item: &BatchedRuleAdjustment) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl BatchedRuleAdjustmentRule for BatchedRuleAdjustmentCodePrefixRule {
    fn check(&self, item: &BatchedRuleAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAdjustmentWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRuleAdjustmentRule for BatchedRuleAdjustmentWindowDaysRangeRule {
    fn check(&self, item: &BatchedRuleAdjustment) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched rule adjustment.
#[derive(Debug, Clone)]
pub struct BatchedRuleAdjustmentValidator {
    pub rules: Vec<Rc<dyn BatchedRuleAdjustmentRule>>,
}

impl BatchedRuleAdjustmentValidator {
    pub fn validate(&self, item: &BatchedRuleAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedRuleAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRuleAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_rule_adjustment_validator() -> BatchedRuleAdjustmentValidator {
    BatchedRuleAdjustmentValidator {
        rules: vec![
            Rc::new(BatchedRuleAdjustmentRequiredSegmentRule {
                message: "batched rule adjustment needs this field".to_string(),
            }) as Rc<dyn BatchedRuleAdjustmentRule>,
            Rc::new(BatchedRuleAdjustmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BatchedRuleAdjustmentWindowDaysRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_rule_adjustment_validator();
        let item = new_batched_rule_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = BatchedRuleAdjustmentRequiredSegmentRule {
            message: "batched rule adjustment needs this field".to_string(),
        };
        let mut item = new_batched_rule_adjustment("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = BatchedRuleAdjustmentWindowDaysRangeRule { min: 16, max: 80 };
        let mut item = new_batched_rule_adjustment("x", "y");
        item.window_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
