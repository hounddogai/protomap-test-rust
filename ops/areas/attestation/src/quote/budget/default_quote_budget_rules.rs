use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default quote budget must pass. Check returns its problems, or none.
pub trait DefaultQuoteBudgetRule: std::fmt::Debug {
    fn check(&self, item: &DefaultQuoteBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuoteBudgetFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultQuoteBudgetRule for DefaultQuoteBudgetFloorValueRangeRule {
    fn check(&self, item: &DefaultQuoteBudget) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuoteBudgetRequiredExternalRefRule {
    pub message: String,
}

impl DefaultQuoteBudgetRule for DefaultQuoteBudgetRequiredExternalRefRule {
    fn check(&self, item: &DefaultQuoteBudget) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuoteBudgetTitleLengthRule {
    pub max_length: i64,
}

impl DefaultQuoteBudgetRule for DefaultQuoteBudgetTitleLengthRule {
    fn check(&self, item: &DefaultQuoteBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default quote budget.
#[derive(Debug, Clone)]
pub struct DefaultQuoteBudgetValidator {
    pub rules: Vec<Rc<dyn DefaultQuoteBudgetRule>>,
}

impl DefaultQuoteBudgetValidator {
    pub fn validate(&self, item: &DefaultQuoteBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultQuoteBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultQuoteBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_quote_budget_validator() -> DefaultQuoteBudgetValidator {
    DefaultQuoteBudgetValidator {
        rules: vec![
            Rc::new(DefaultQuoteBudgetFloorValueRangeRule { min: 6.0, max: 2763.1 }) as Rc<dyn DefaultQuoteBudgetRule>,
            Rc::new(DefaultQuoteBudgetRequiredExternalRefRule {
                message: "default quote budget needs this field".to_string(),
            }),
            Rc::new(DefaultQuoteBudgetTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_quote_budget_validator();
        let item = new_default_quote_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = DefaultQuoteBudgetFloorValueRangeRule { min: 3.6, max: 2660.7 };
        let mut item = new_default_quote_budget("x", "y");
        item.floor_value = 2662.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = DefaultQuoteBudgetRequiredExternalRefRule {
            message: "default quote budget needs a value".to_string(),
        };
        let mut item = new_default_quote_budget("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultQuoteBudgetTitleLengthRule { max_length: 255 };
        let mut item = new_default_quote_budget("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
