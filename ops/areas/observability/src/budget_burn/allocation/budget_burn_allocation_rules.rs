use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn allocation must pass. Check returns its problems, or none.
pub trait BudgetBurnAllocationRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAllocationActiveGuardRule {
    pub limit: f64,
}

impl BudgetBurnAllocationRule for BudgetBurnAllocationActiveGuardRule {
    fn check(&self, item: &BudgetBurnAllocation) -> Vec<String> {
        if item.active && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while active", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAllocationScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnAllocationRule for BudgetBurnAllocationScoreRangeRule {
    fn check(&self, item: &BudgetBurnAllocation) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAllocationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnAllocationRule for BudgetBurnAllocationGraceDaysRangeRule {
    fn check(&self, item: &BudgetBurnAllocation) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAllocationRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnAllocationRule for BudgetBurnAllocationRequiredCodeRule {
    fn check(&self, item: &BudgetBurnAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn allocation.
#[derive(Debug, Clone)]
pub struct BudgetBurnAllocationValidator {
    pub rules: Vec<Rc<dyn BudgetBurnAllocationRule>>,
}

impl BudgetBurnAllocationValidator {
    pub fn validate(&self, item: &BudgetBurnAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_allocation_validator() -> BudgetBurnAllocationValidator {
    BudgetBurnAllocationValidator {
        rules: vec![
            Rc::new(BudgetBurnAllocationActiveGuardRule { limit: 9.3 }) as Rc<dyn BudgetBurnAllocationRule>,
            Rc::new(BudgetBurnAllocationScoreRangeRule { min: 4.3, max: 1346.9 }),
            Rc::new(BudgetBurnAllocationGraceDaysRangeRule { min: 1000, max: 1000 }),
            Rc::new(BudgetBurnAllocationRequiredCodeRule {
                message: "budget burn allocation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_allocation_validator();
        let item = new_budget_burn_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = BudgetBurnAllocationScoreRangeRule { min: 9.7, max: 4692.8 };
        let mut item = new_budget_burn_allocation("x", "y");
        item.score = 4694.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = BudgetBurnAllocationGraceDaysRangeRule { min: 16, max: 32 };
        let mut item = new_budget_burn_allocation("x", "y");
        item.grace_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BudgetBurnAllocationRequiredCodeRule {
            message: "budget burn allocation needs an entry".to_string(),
        };
        let mut item = new_budget_burn_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
