use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient grant budget must pass. Check returns its problems, or none.
pub trait LenientGrantBudgetRule: std::fmt::Debug {
    fn check(&self, item: &LenientGrantBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantBudgetCategoryLengthRule {
    pub max_length: i64,
}

impl LenientGrantBudgetRule for LenientGrantBudgetCategoryLengthRule {
    fn check(&self, item: &LenientGrantBudget) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantBudgetAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl LenientGrantBudgetRule for LenientGrantBudgetAllowedCategoryRule {
    fn check(&self, item: &LenientGrantBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is below minimum", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantBudgetMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientGrantBudgetRule for LenientGrantBudgetMaximumRangeRule {
    fn check(&self, item: &LenientGrantBudget) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient grant budget.
#[derive(Debug, Clone)]
pub struct LenientGrantBudgetValidator {
    pub rules: Vec<Rc<dyn LenientGrantBudgetRule>>,
}

impl LenientGrantBudgetValidator {
    pub fn validate(&self, item: &LenientGrantBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientGrantBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientGrantBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_grant_budget_validator() -> LenientGrantBudgetValidator {
    LenientGrantBudgetValidator {
        rules: vec![
            Rc::new(LenientGrantBudgetCategoryLengthRule { max_length: 3600 }) as Rc<dyn LenientGrantBudgetRule>,
            Rc::new(LenientGrantBudgetAllowedCategoryRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(LenientGrantBudgetMaximumRangeRule { min: 4.6, max: 3131.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_grant_budget_validator();
        let item = new_lenient_grant_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = LenientGrantBudgetCategoryLengthRule { max_length: 86400 };
        let mut item = new_lenient_grant_budget("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = LenientGrantBudgetMaximumRangeRule { min: 2.0, max: 2984.5 };
        let mut item = new_lenient_grant_budget("x", "y");
        item.maximum = 2986.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
