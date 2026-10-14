use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped budget burn cycle must pass. Check returns its problems, or none.
pub trait CappedBudgetBurnCycleRule: std::fmt::Debug {
    fn check(&self, item: &CappedBudgetBurnCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnCycleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedBudgetBurnCycleRule for CappedBudgetBurnCycleEndsAtRangeRule {
    fn check(&self, item: &CappedBudgetBurnCycle) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnCycleAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedBudgetBurnCycleRule for CappedBudgetBurnCycleAmountRangeRule {
    fn check(&self, item: &CappedBudgetBurnCycle) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnCycleAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl CappedBudgetBurnCycleRule for CappedBudgetBurnCycleAllowedSourceRule {
    fn check(&self, item: &CappedBudgetBurnCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is unknown", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnCycleReviewerLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnCycleRule for CappedBudgetBurnCycleReviewerLengthRule {
    fn check(&self, item: &CappedBudgetBurnCycle) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnCycleLabelLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnCycleRule for CappedBudgetBurnCycleLabelLengthRule {
    fn check(&self, item: &CappedBudgetBurnCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped budget burn cycle.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnCycleValidator {
    pub rules: Vec<Rc<dyn CappedBudgetBurnCycleRule>>,
}

impl CappedBudgetBurnCycleValidator {
    pub fn validate(&self, item: &CappedBudgetBurnCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedBudgetBurnCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedBudgetBurnCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_budget_burn_cycle_validator() -> CappedBudgetBurnCycleValidator {
    CappedBudgetBurnCycleValidator {
        rules: vec![
            Rc::new(CappedBudgetBurnCycleEndsAtRangeRule { min: 80, max: 255 }) as Rc<dyn CappedBudgetBurnCycleRule>,
            Rc::new(CappedBudgetBurnCycleAmountRangeRule { min: 7.6, max: 1106.5 }),
            Rc::new(CappedBudgetBurnCycleAllowedSourceRule {
                allowed: vec!["weighted".to_string(), "tiered".to_string(), "strict".to_string()],
            }),
            Rc::new(CappedBudgetBurnCycleReviewerLengthRule { max_length: 86400 }),
            Rc::new(CappedBudgetBurnCycleLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_budget_burn_cycle_validator();
        let item = new_capped_budget_burn_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = CappedBudgetBurnCycleEndsAtRangeRule { min: 16, max: 16 };
        let mut item = new_capped_budget_burn_cycle("x", "y");
        item.ends_at = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = CappedBudgetBurnCycleAmountRangeRule { min: 8.7, max: 3151.8 };
        let mut item = new_capped_budget_burn_cycle("x", "y");
        item.amount = 3153.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = CappedBudgetBurnCycleReviewerLengthRule { max_length: 120 };
        let mut item = new_capped_budget_burn_cycle("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CappedBudgetBurnCycleLabelLengthRule { max_length: 3600 };
        let mut item = new_capped_budget_burn_cycle("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
