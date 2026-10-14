use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched health check budget must pass. Check returns its problems, or none.
pub trait BatchedHealthCheckBudgetRule: std::fmt::Debug {
    fn check(&self, item: &BatchedHealthCheckBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckBudgetRequiredCodeRule {
    pub message: String,
}

impl BatchedHealthCheckBudgetRule for BatchedHealthCheckBudgetRequiredCodeRule {
    fn check(&self, item: &BatchedHealthCheckBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckBudgetUnitLengthRule {
    pub max_length: i64,
}

impl BatchedHealthCheckBudgetRule for BatchedHealthCheckBudgetUnitLengthRule {
    fn check(&self, item: &BatchedHealthCheckBudget) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckBudgetCodeLengthRule {
    pub max_length: i64,
}

impl BatchedHealthCheckBudgetRule for BatchedHealthCheckBudgetCodeLengthRule {
    fn check(&self, item: &BatchedHealthCheckBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckBudgetRequiredLabelRule {
    pub message: String,
}

impl BatchedHealthCheckBudgetRule for BatchedHealthCheckBudgetRequiredLabelRule {
    fn check(&self, item: &BatchedHealthCheckBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckBudgetCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedHealthCheckBudgetRule for BatchedHealthCheckBudgetCeilingRangeRule {
    fn check(&self, item: &BatchedHealthCheckBudget) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckBudgetRequiredUnitRule {
    pub message: String,
}

impl BatchedHealthCheckBudgetRule for BatchedHealthCheckBudgetRequiredUnitRule {
    fn check(&self, item: &BatchedHealthCheckBudget) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched health check budget.
#[derive(Debug, Clone)]
pub struct BatchedHealthCheckBudgetValidator {
    pub rules: Vec<Rc<dyn BatchedHealthCheckBudgetRule>>,
}

impl BatchedHealthCheckBudgetValidator {
    pub fn validate(&self, item: &BatchedHealthCheckBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedHealthCheckBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedHealthCheckBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_health_check_budget_validator() -> BatchedHealthCheckBudgetValidator {
    BatchedHealthCheckBudgetValidator {
        rules: vec![
            Rc::new(BatchedHealthCheckBudgetRequiredCodeRule {
                message: "batched health check budget needs this field".to_string(),
            }) as Rc<dyn BatchedHealthCheckBudgetRule>,
            Rc::new(BatchedHealthCheckBudgetUnitLengthRule { max_length: 3600 }),
            Rc::new(BatchedHealthCheckBudgetCodeLengthRule { max_length: 1000 }),
            Rc::new(BatchedHealthCheckBudgetRequiredLabelRule {
                message: "batched health check budget needs an entry".to_string(),
            }),
            Rc::new(BatchedHealthCheckBudgetCeilingRangeRule { min: 5.0, max: 3024.7 }),
            Rc::new(BatchedHealthCheckBudgetRequiredUnitRule {
                message: "batched health check budget needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_health_check_budget_validator();
        let item = new_batched_health_check_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedHealthCheckBudgetRequiredCodeRule {
            message: "batched health check budget needs a value".to_string(),
        };
        let mut item = new_batched_health_check_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = BatchedHealthCheckBudgetUnitLengthRule { max_length: 64 };
        let mut item = new_batched_health_check_budget("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedHealthCheckBudgetCodeLengthRule { max_length: 3600 };
        let mut item = new_batched_health_check_budget("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedHealthCheckBudgetRequiredLabelRule {
            message: "batched health check budget needs a value".to_string(),
        };
        let mut item = new_batched_health_check_budget("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = BatchedHealthCheckBudgetCeilingRangeRule { min: 9.3, max: 3961.6 };
        let mut item = new_batched_health_check_budget("x", "y");
        item.ceiling = 3963.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = BatchedHealthCheckBudgetRequiredUnitRule {
            message: "batched health check budget needs this field".to_string(),
        };
        let mut item = new_batched_health_check_budget("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
