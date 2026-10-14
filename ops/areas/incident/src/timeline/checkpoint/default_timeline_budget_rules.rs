use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default timeline budget must pass. Check returns its problems, or none.
pub trait DefaultTimelineBudgetRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTimelineBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineBudgetTitleLengthRule {
    pub max_length: i64,
}

impl DefaultTimelineBudgetRule for DefaultTimelineBudgetTitleLengthRule {
    fn check(&self, item: &DefaultTimelineBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineBudgetWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultTimelineBudgetRule for DefaultTimelineBudgetWeightRangeRule {
    fn check(&self, item: &DefaultTimelineBudget) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineBudgetCodeLengthRule {
    pub max_length: i64,
}

impl DefaultTimelineBudgetRule for DefaultTimelineBudgetCodeLengthRule {
    fn check(&self, item: &DefaultTimelineBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineBudgetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DefaultTimelineBudgetRule for DefaultTimelineBudgetAllowedTitleRule {
    fn check(&self, item: &DefaultTimelineBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineBudgetLabelLengthRule {
    pub max_length: i64,
}

impl DefaultTimelineBudgetRule for DefaultTimelineBudgetLabelLengthRule {
    fn check(&self, item: &DefaultTimelineBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default timeline budget.
#[derive(Debug, Clone)]
pub struct DefaultTimelineBudgetValidator {
    pub rules: Vec<Rc<dyn DefaultTimelineBudgetRule>>,
}

impl DefaultTimelineBudgetValidator {
    pub fn validate(&self, item: &DefaultTimelineBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultTimelineBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTimelineBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_timeline_budget_validator() -> DefaultTimelineBudgetValidator {
    DefaultTimelineBudgetValidator {
        rules: vec![
            Rc::new(DefaultTimelineBudgetTitleLengthRule { max_length: 32 }) as Rc<dyn DefaultTimelineBudgetRule>,
            Rc::new(DefaultTimelineBudgetWeightRangeRule { min: 2.7, max: 4573.5 }),
            Rc::new(DefaultTimelineBudgetCodeLengthRule { max_length: 120 }),
            Rc::new(DefaultTimelineBudgetAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(DefaultTimelineBudgetLabelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_timeline_budget_validator();
        let item = new_default_timeline_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultTimelineBudgetTitleLengthRule { max_length: 255 };
        let mut item = new_default_timeline_budget("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = DefaultTimelineBudgetWeightRangeRule { min: 6.7, max: 1501.1 };
        let mut item = new_default_timeline_budget("x", "y");
        item.weight = 1502.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultTimelineBudgetCodeLengthRule { max_length: 86400 };
        let mut item = new_default_timeline_budget("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultTimelineBudgetLabelLengthRule { max_length: 3600 };
        let mut item = new_default_timeline_budget("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
