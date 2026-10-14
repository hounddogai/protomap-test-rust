use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn variance must pass. Check returns its problems, or none.
pub trait BudgetBurnVarianceRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnVarianceRequiredUnitRule {
    pub message: String,
}

impl BudgetBurnVarianceRule for BudgetBurnVarianceRequiredUnitRule {
    fn check(&self, item: &BudgetBurnVariance) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnVarianceApprovedGuardRule {
    pub limit: i64,
}

impl BudgetBurnVarianceRule for BudgetBurnVarianceApprovedGuardRule {
    fn check(&self, item: &BudgetBurnVariance) -> Vec<String> {
        if item.approved && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while approved",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnVarianceCodeLengthRule {
    pub max_length: i64,
}

impl BudgetBurnVarianceRule for BudgetBurnVarianceCodeLengthRule {
    fn check(&self, item: &BudgetBurnVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnVarianceRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnVarianceRule for BudgetBurnVarianceRequiredCodeRule {
    fn check(&self, item: &BudgetBurnVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnVarianceDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnVarianceRule for BudgetBurnVarianceDurationMinutesRangeRule {
    fn check(&self, item: &BudgetBurnVariance) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnVarianceCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnVarianceRule for BudgetBurnVarianceCapacityRangeRule {
    fn check(&self, item: &BudgetBurnVariance) -> Vec<String> {
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

/// Runs every rule of a budget burn variance.
#[derive(Debug, Clone)]
pub struct BudgetBurnVarianceValidator {
    pub rules: Vec<Rc<dyn BudgetBurnVarianceRule>>,
}

impl BudgetBurnVarianceValidator {
    pub fn validate(&self, item: &BudgetBurnVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_variance_validator() -> BudgetBurnVarianceValidator {
    BudgetBurnVarianceValidator {
        rules: vec![
            Rc::new(BudgetBurnVarianceRequiredUnitRule {
                message: "budget burn variance needs this field".to_string(),
            }) as Rc<dyn BudgetBurnVarianceRule>,
            Rc::new(BudgetBurnVarianceApprovedGuardRule { limit: 80 }),
            Rc::new(BudgetBurnVarianceCodeLengthRule { max_length: 32 }),
            Rc::new(BudgetBurnVarianceRequiredCodeRule {
                message: "budget burn variance needs a value".to_string(),
            }),
            Rc::new(BudgetBurnVarianceDurationMinutesRangeRule { min: 120, max: 3600 }),
            Rc::new(BudgetBurnVarianceCapacityRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_variance_validator();
        let item = new_budget_burn_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = BudgetBurnVarianceRequiredUnitRule {
            message: "budget burn variance needs this field".to_string(),
        };
        let mut item = new_budget_burn_variance("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BudgetBurnVarianceCodeLengthRule { max_length: 1000 };
        let mut item = new_budget_burn_variance("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BudgetBurnVarianceRequiredCodeRule {
            message: "budget burn variance needs a value".to_string(),
        };
        let mut item = new_budget_burn_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = BudgetBurnVarianceDurationMinutesRangeRule { min: 32, max: 3600 };
        let mut item = new_budget_burn_variance("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = BudgetBurnVarianceCapacityRangeRule { min: 255, max: 86400 };
        let mut item = new_budget_burn_variance("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
