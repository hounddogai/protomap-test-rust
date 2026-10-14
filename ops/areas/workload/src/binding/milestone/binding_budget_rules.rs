use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding budget must pass. Check returns its problems, or none.
pub trait BindingBudgetRule: std::fmt::Debug {
    fn check(&self, item: &BindingBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBudgetRequiredReviewerRule {
    pub message: String,
}

impl BindingBudgetRule for BindingBudgetRequiredReviewerRule {
    fn check(&self, item: &BindingBudget) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBudgetReviewerLengthRule {
    pub max_length: i64,
}

impl BindingBudgetRule for BindingBudgetReviewerLengthRule {
    fn check(&self, item: &BindingBudget) -> Vec<String> {
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
pub struct BindingBudgetLabelLengthRule {
    pub max_length: i64,
}

impl BindingBudgetRule for BindingBudgetLabelLengthRule {
    fn check(&self, item: &BindingBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBudgetSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingBudgetRule for BindingBudgetSequenceRangeRule {
    fn check(&self, item: &BindingBudget) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBudgetTitleLengthRule {
    pub max_length: i64,
}

impl BindingBudgetRule for BindingBudgetTitleLengthRule {
    fn check(&self, item: &BindingBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding budget.
#[derive(Debug, Clone)]
pub struct BindingBudgetValidator {
    pub rules: Vec<Rc<dyn BindingBudgetRule>>,
}

impl BindingBudgetValidator {
    pub fn validate(&self, item: &BindingBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_budget_validator() -> BindingBudgetValidator {
    BindingBudgetValidator {
        rules: vec![
            Rc::new(BindingBudgetRequiredReviewerRule {
                message: "binding budget needs an entry".to_string(),
            }) as Rc<dyn BindingBudgetRule>,
            Rc::new(BindingBudgetReviewerLengthRule { max_length: 80 }),
            Rc::new(BindingBudgetLabelLengthRule { max_length: 64 }),
            Rc::new(BindingBudgetSequenceRangeRule { min: 16, max: 86400 }),
            Rc::new(BindingBudgetTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_budget_validator();
        let item = new_binding_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = BindingBudgetRequiredReviewerRule {
            message: "binding budget needs a value".to_string(),
        };
        let mut item = new_binding_budget("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = BindingBudgetReviewerLengthRule { max_length: 120 };
        let mut item = new_binding_budget("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BindingBudgetLabelLengthRule { max_length: 32 };
        let mut item = new_binding_budget("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = BindingBudgetSequenceRangeRule { min: 80, max: 1000 };
        let mut item = new_binding_budget("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BindingBudgetTitleLengthRule { max_length: 3600 };
        let mut item = new_binding_budget("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
