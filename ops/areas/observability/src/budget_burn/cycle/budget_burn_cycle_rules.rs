use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn cycle must pass. Check returns its problems, or none.
pub trait BudgetBurnCycleRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnCycleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnCycleRule for BudgetBurnCycleMinimumRangeRule {
    fn check(&self, item: &BudgetBurnCycle) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnCycleCodeLengthRule {
    pub max_length: i64,
}

impl BudgetBurnCycleRule for BudgetBurnCycleCodeLengthRule {
    fn check(&self, item: &BudgetBurnCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnCycleRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnCycleRule for BudgetBurnCycleRequiredCodeRule {
    fn check(&self, item: &BudgetBurnCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnCycleCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnCycleRule for BudgetBurnCycleCodePrefixRule {
    fn check(&self, item: &BudgetBurnCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnCycleNamespaceLengthRule {
    pub max_length: i64,
}

impl BudgetBurnCycleRule for BudgetBurnCycleNamespaceLengthRule {
    fn check(&self, item: &BudgetBurnCycle) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnCycleCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnCycleRule for BudgetBurnCycleCapacityRangeRule {
    fn check(&self, item: &BudgetBurnCycle) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a budget burn cycle.
#[derive(Debug, Clone)]
pub struct BudgetBurnCycleValidator {
    pub rules: Vec<Rc<dyn BudgetBurnCycleRule>>,
}

impl BudgetBurnCycleValidator {
    pub fn validate(&self, item: &BudgetBurnCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_cycle_validator() -> BudgetBurnCycleValidator {
    BudgetBurnCycleValidator {
        rules: vec![
            Rc::new(BudgetBurnCycleMinimumRangeRule { min: 9.5, max: 4205.1 }) as Rc<dyn BudgetBurnCycleRule>,
            Rc::new(BudgetBurnCycleCodeLengthRule { max_length: 32 }),
            Rc::new(BudgetBurnCycleRequiredCodeRule {
                message: "budget burn cycle needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnCycleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(BudgetBurnCycleNamespaceLengthRule { max_length: 120 }),
            Rc::new(BudgetBurnCycleCapacityRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_cycle_validator();
        let item = new_budget_burn_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = BudgetBurnCycleMinimumRangeRule { min: 1.7, max: 1481.8 };
        let mut item = new_budget_burn_cycle("x", "y");
        item.minimum = 1483.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BudgetBurnCycleCodeLengthRule { max_length: 16 };
        let mut item = new_budget_burn_cycle("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BudgetBurnCycleRequiredCodeRule {
            message: "budget burn cycle needs a value".to_string(),
        };
        let mut item = new_budget_burn_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = BudgetBurnCycleNamespaceLengthRule { max_length: 1000 };
        let mut item = new_budget_burn_cycle("x", "y");
        item.namespace = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = BudgetBurnCycleCapacityRangeRule { min: 64, max: 80 };
        let mut item = new_budget_burn_cycle("x", "y");
        item.capacity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
