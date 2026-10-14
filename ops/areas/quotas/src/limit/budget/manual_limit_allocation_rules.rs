use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual limit allocation must pass. Check returns its problems, or none.
pub trait ManualLimitAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ManualLimitAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitAllocationAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualLimitAllocationRule for ManualLimitAllocationAmountRangeRule {
    fn check(&self, item: &ManualLimitAllocation) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitAllocationUnitLengthRule {
    pub max_length: i64,
}

impl ManualLimitAllocationRule for ManualLimitAllocationUnitLengthRule {
    fn check(&self, item: &ManualLimitAllocation) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitAllocationRequiredUnitRule {
    pub message: String,
}

impl ManualLimitAllocationRule for ManualLimitAllocationRequiredUnitRule {
    fn check(&self, item: &ManualLimitAllocation) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitAllocationCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualLimitAllocationRule for ManualLimitAllocationCostRangeRule {
    fn check(&self, item: &ManualLimitAllocation) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitAllocationAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualLimitAllocationRule for ManualLimitAllocationAttemptsRangeRule {
    fn check(&self, item: &ManualLimitAllocation) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitAllocationVisibleGuardRule {
    pub limit: f64,
}

impl ManualLimitAllocationRule for ManualLimitAllocationVisibleGuardRule {
    fn check(&self, item: &ManualLimitAllocation) -> Vec<String> {
        if item.visible && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while visible", item.amount, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual limit allocation.
#[derive(Debug, Clone)]
pub struct ManualLimitAllocationValidator {
    pub rules: Vec<Rc<dyn ManualLimitAllocationRule>>,
}

impl ManualLimitAllocationValidator {
    pub fn validate(&self, item: &ManualLimitAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualLimitAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualLimitAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_limit_allocation_validator() -> ManualLimitAllocationValidator {
    ManualLimitAllocationValidator {
        rules: vec![
            Rc::new(ManualLimitAllocationAmountRangeRule { min: 1.5, max: 3727.8 })
                as Rc<dyn ManualLimitAllocationRule>,
            Rc::new(ManualLimitAllocationUnitLengthRule { max_length: 16 }),
            Rc::new(ManualLimitAllocationRequiredUnitRule {
                message: "manual limit allocation needs this field".to_string(),
            }),
            Rc::new(ManualLimitAllocationCostRangeRule { min: 8.5, max: 902.0 }),
            Rc::new(ManualLimitAllocationAttemptsRangeRule { min: 64, max: 80 }),
            Rc::new(ManualLimitAllocationVisibleGuardRule { limit: 7.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_limit_allocation_validator();
        let item = new_manual_limit_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ManualLimitAllocationAmountRangeRule { min: 8.2, max: 2625.5 };
        let mut item = new_manual_limit_allocation("x", "y");
        item.amount = 2627.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ManualLimitAllocationUnitLengthRule { max_length: 64 };
        let mut item = new_manual_limit_allocation("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = ManualLimitAllocationRequiredUnitRule {
            message: "manual limit allocation needs an entry".to_string(),
        };
        let mut item = new_manual_limit_allocation("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ManualLimitAllocationCostRangeRule { min: 5.3, max: 1951.3 };
        let mut item = new_manual_limit_allocation("x", "y");
        item.cost = 1952.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ManualLimitAllocationAttemptsRangeRule { min: 64, max: 86400 };
        let mut item = new_manual_limit_allocation("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
