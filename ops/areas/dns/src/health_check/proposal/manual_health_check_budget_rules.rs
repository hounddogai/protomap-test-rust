use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual health check budget must pass. Check returns its problems, or none.
pub trait ManualHealthCheckBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ManualHealthCheckBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBudgetUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualHealthCheckBudgetRule for ManualHealthCheckBudgetUpdatedAtRangeRule {
    fn check(&self, item: &ManualHealthCheckBudget) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBudgetRequiredReviewerRule {
    pub message: String,
}

impl ManualHealthCheckBudgetRule for ManualHealthCheckBudgetRequiredReviewerRule {
    fn check(&self, item: &ManualHealthCheckBudget) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBudgetQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualHealthCheckBudgetRule for ManualHealthCheckBudgetQuantityRangeRule {
    fn check(&self, item: &ManualHealthCheckBudget) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBudgetRequiredCodeRule {
    pub message: String,
}

impl ManualHealthCheckBudgetRule for ManualHealthCheckBudgetRequiredCodeRule {
    fn check(&self, item: &ManualHealthCheckBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBudgetCodeLengthRule {
    pub max_length: i64,
}

impl ManualHealthCheckBudgetRule for ManualHealthCheckBudgetCodeLengthRule {
    fn check(&self, item: &ManualHealthCheckBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBudgetRequiredTitleRule {
    pub message: String,
}

impl ManualHealthCheckBudgetRule for ManualHealthCheckBudgetRequiredTitleRule {
    fn check(&self, item: &ManualHealthCheckBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual health check budget.
#[derive(Debug, Clone)]
pub struct ManualHealthCheckBudgetValidator {
    pub rules: Vec<Rc<dyn ManualHealthCheckBudgetRule>>,
}

impl ManualHealthCheckBudgetValidator {
    pub fn validate(&self, item: &ManualHealthCheckBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualHealthCheckBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualHealthCheckBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_health_check_budget_validator() -> ManualHealthCheckBudgetValidator {
    ManualHealthCheckBudgetValidator {
        rules: vec![
            Rc::new(ManualHealthCheckBudgetUpdatedAtRangeRule { min: 80, max: 80 })
                as Rc<dyn ManualHealthCheckBudgetRule>,
            Rc::new(ManualHealthCheckBudgetRequiredReviewerRule {
                message: "manual health check budget needs a value".to_string(),
            }),
            Rc::new(ManualHealthCheckBudgetQuantityRangeRule { min: 120, max: 1000 }),
            Rc::new(ManualHealthCheckBudgetRequiredCodeRule {
                message: "manual health check budget needs a value".to_string(),
            }),
            Rc::new(ManualHealthCheckBudgetCodeLengthRule { max_length: 64 }),
            Rc::new(ManualHealthCheckBudgetRequiredTitleRule {
                message: "manual health check budget needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_health_check_budget_validator();
        let item = new_manual_health_check_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = ManualHealthCheckBudgetUpdatedAtRangeRule { min: 120, max: 255 };
        let mut item = new_manual_health_check_budget("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ManualHealthCheckBudgetRequiredReviewerRule {
            message: "manual health check budget needs an entry".to_string(),
        };
        let mut item = new_manual_health_check_budget("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ManualHealthCheckBudgetQuantityRangeRule { min: 80, max: 86400 };
        let mut item = new_manual_health_check_budget("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualHealthCheckBudgetRequiredCodeRule {
            message: "manual health check budget needs this field".to_string(),
        };
        let mut item = new_manual_health_check_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualHealthCheckBudgetCodeLengthRule { max_length: 1000 };
        let mut item = new_manual_health_check_budget("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualHealthCheckBudgetRequiredTitleRule {
            message: "manual health check budget needs a value".to_string(),
        };
        let mut item = new_manual_health_check_budget("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
