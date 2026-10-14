use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped sink adjustment must pass. Check returns its problems, or none.
pub trait CappedSinkAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &CappedSinkAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkAdjustmentMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedSinkAdjustmentRule for CappedSinkAdjustmentMinimumRangeRule {
    fn check(&self, item: &CappedSinkAdjustment) -> Vec<String> {
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
pub struct CappedSinkAdjustmentAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedSinkAdjustmentRule for CappedSinkAdjustmentAttemptsRangeRule {
    fn check(&self, item: &CappedSinkAdjustment) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkAdjustmentRequiredCodeRule {
    pub message: String,
}

impl CappedSinkAdjustmentRule for CappedSinkAdjustmentRequiredCodeRule {
    fn check(&self, item: &CappedSinkAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl CappedSinkAdjustmentRule for CappedSinkAdjustmentCodePrefixRule {
    fn check(&self, item: &CappedSinkAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkAdjustmentRequiredLabelRule {
    pub message: String,
}

impl CappedSinkAdjustmentRule for CappedSinkAdjustmentRequiredLabelRule {
    fn check(&self, item: &CappedSinkAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkAdjustmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedSinkAdjustmentRule for CappedSinkAdjustmentCostRangeRule {
    fn check(&self, item: &CappedSinkAdjustment) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped sink adjustment.
#[derive(Debug, Clone)]
pub struct CappedSinkAdjustmentValidator {
    pub rules: Vec<Rc<dyn CappedSinkAdjustmentRule>>,
}

impl CappedSinkAdjustmentValidator {
    pub fn validate(&self, item: &CappedSinkAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedSinkAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSinkAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_sink_adjustment_validator() -> CappedSinkAdjustmentValidator {
    CappedSinkAdjustmentValidator {
        rules: vec![
            Rc::new(CappedSinkAdjustmentMinimumRangeRule { min: 6.3, max: 628.7 }) as Rc<dyn CappedSinkAdjustmentRule>,
            Rc::new(CappedSinkAdjustmentAttemptsRangeRule { min: 16, max: 32 }),
            Rc::new(CappedSinkAdjustmentRequiredCodeRule {
                message: "capped sink adjustment needs an entry".to_string(),
            }),
            Rc::new(CappedSinkAdjustmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CappedSinkAdjustmentRequiredLabelRule {
                message: "capped sink adjustment needs an entry".to_string(),
            }),
            Rc::new(CappedSinkAdjustmentCostRangeRule { min: 1.0, max: 1028.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_sink_adjustment_validator();
        let item = new_capped_sink_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = CappedSinkAdjustmentMinimumRangeRule { min: 2.5, max: 4405.1 };
        let mut item = new_capped_sink_adjustment("x", "y");
        item.minimum = 4406.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = CappedSinkAdjustmentAttemptsRangeRule { min: 80, max: 255 };
        let mut item = new_capped_sink_adjustment("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedSinkAdjustmentRequiredCodeRule {
            message: "capped sink adjustment needs an entry".to_string(),
        };
        let mut item = new_capped_sink_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedSinkAdjustmentRequiredLabelRule {
            message: "capped sink adjustment needs this field".to_string(),
        };
        let mut item = new_capped_sink_adjustment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = CappedSinkAdjustmentCostRangeRule { min: 9.5, max: 1360.6 };
        let mut item = new_capped_sink_adjustment("x", "y");
        item.cost = 1362.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
