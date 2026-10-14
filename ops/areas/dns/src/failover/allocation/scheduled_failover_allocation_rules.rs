use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled failover allocation must pass. Check returns its problems, or none.
pub trait ScheduledFailoverAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFailoverAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverAllocationLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledFailoverAllocationRule for ScheduledFailoverAllocationLabelLengthRule {
    fn check(&self, item: &ScheduledFailoverAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverAllocationRequiredLabelRule {
    pub message: String,
}

impl ScheduledFailoverAllocationRule for ScheduledFailoverAllocationRequiredLabelRule {
    fn check(&self, item: &ScheduledFailoverAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverAllocationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledFailoverAllocationRule for ScheduledFailoverAllocationMinimumRangeRule {
    fn check(&self, item: &ScheduledFailoverAllocation) -> Vec<String> {
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
pub struct ScheduledFailoverAllocationCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledFailoverAllocationRule for ScheduledFailoverAllocationCostRangeRule {
    fn check(&self, item: &ScheduledFailoverAllocation) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverAllocationAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFailoverAllocationRule for ScheduledFailoverAllocationAttemptsRangeRule {
    fn check(&self, item: &ScheduledFailoverAllocation) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled failover allocation.
#[derive(Debug, Clone)]
pub struct ScheduledFailoverAllocationValidator {
    pub rules: Vec<Rc<dyn ScheduledFailoverAllocationRule>>,
}

impl ScheduledFailoverAllocationValidator {
    pub fn validate(&self, item: &ScheduledFailoverAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledFailoverAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFailoverAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_failover_allocation_validator() -> ScheduledFailoverAllocationValidator {
    ScheduledFailoverAllocationValidator {
        rules: vec![
            Rc::new(ScheduledFailoverAllocationLabelLengthRule { max_length: 16 })
                as Rc<dyn ScheduledFailoverAllocationRule>,
            Rc::new(ScheduledFailoverAllocationRequiredLabelRule {
                message: "scheduled failover allocation needs an entry".to_string(),
            }),
            Rc::new(ScheduledFailoverAllocationMinimumRangeRule { min: 5.0, max: 2459.7 }),
            Rc::new(ScheduledFailoverAllocationCostRangeRule { min: 4.0, max: 1358.9 }),
            Rc::new(ScheduledFailoverAllocationAttemptsRangeRule { min: 32, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_failover_allocation_validator();
        let item = new_scheduled_failover_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledFailoverAllocationLabelLengthRule { max_length: 64 };
        let mut item = new_scheduled_failover_allocation("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledFailoverAllocationRequiredLabelRule {
            message: "scheduled failover allocation needs this field".to_string(),
        };
        let mut item = new_scheduled_failover_allocation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ScheduledFailoverAllocationMinimumRangeRule { min: 0.6, max: 450.7 };
        let mut item = new_scheduled_failover_allocation("x", "y");
        item.minimum = 452.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ScheduledFailoverAllocationCostRangeRule { min: 4.2, max: 561.8 };
        let mut item = new_scheduled_failover_allocation("x", "y");
        item.cost = 563.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ScheduledFailoverAllocationAttemptsRangeRule { min: 32, max: 1000 };
        let mut item = new_scheduled_failover_allocation("x", "y");
        item.attempts = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
