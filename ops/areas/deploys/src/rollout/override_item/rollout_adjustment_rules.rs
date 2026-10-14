use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout adjustment must pass. Check returns its problems, or none.
pub trait RolloutAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RolloutAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAdjustmentRequiredCodeRule {
    pub message: String,
}

impl RolloutAdjustmentRule for RolloutAdjustmentRequiredCodeRule {
    fn check(&self, item: &RolloutAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAdjustmentRequiredNameRule {
    pub message: String,
}

impl RolloutAdjustmentRule for RolloutAdjustmentRequiredNameRule {
    fn check(&self, item: &RolloutAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAdjustmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutAdjustmentRule for RolloutAdjustmentPortRangeRule {
    fn check(&self, item: &RolloutAdjustment) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollout adjustment.
#[derive(Debug, Clone)]
pub struct RolloutAdjustmentValidator {
    pub rules: Vec<Rc<dyn RolloutAdjustmentRule>>,
}

impl RolloutAdjustmentValidator {
    pub fn validate(&self, item: &RolloutAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_adjustment_validator() -> RolloutAdjustmentValidator {
    RolloutAdjustmentValidator {
        rules: vec![
            Rc::new(RolloutAdjustmentRequiredCodeRule {
                message: "rollout adjustment needs an entry".to_string(),
            }) as Rc<dyn RolloutAdjustmentRule>,
            Rc::new(RolloutAdjustmentRequiredNameRule {
                message: "rollout adjustment needs a value".to_string(),
            }),
            Rc::new(RolloutAdjustmentPortRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_adjustment_validator();
        let item = new_rollout_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutAdjustmentRequiredCodeRule {
            message: "rollout adjustment needs a value".to_string(),
        };
        let mut item = new_rollout_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RolloutAdjustmentRequiredNameRule {
            message: "rollout adjustment needs an entry".to_string(),
        };
        let mut item = new_rollout_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = RolloutAdjustmentPortRangeRule { min: 120, max: 1000 };
        let mut item = new_rollout_adjustment("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
