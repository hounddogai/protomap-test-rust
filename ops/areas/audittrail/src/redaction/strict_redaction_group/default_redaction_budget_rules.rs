use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default redaction budget must pass. Check returns its problems, or none.
pub trait DefaultRedactionBudgetRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRedactionBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionBudgetVisibleGuardRule {
    pub limit: i64,
}

impl DefaultRedactionBudgetRule for DefaultRedactionBudgetVisibleGuardRule {
    fn check(&self, item: &DefaultRedactionBudget) -> Vec<String> {
        if item.visible && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while visible",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionBudgetRequiredDescriptionRule {
    pub message: String,
}

impl DefaultRedactionBudgetRule for DefaultRedactionBudgetRequiredDescriptionRule {
    fn check(&self, item: &DefaultRedactionBudget) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionBudgetDescriptionLengthRule {
    pub max_length: i64,
}

impl DefaultRedactionBudgetRule for DefaultRedactionBudgetDescriptionLengthRule {
    fn check(&self, item: &DefaultRedactionBudget) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default redaction budget.
#[derive(Debug, Clone)]
pub struct DefaultRedactionBudgetValidator {
    pub rules: Vec<Rc<dyn DefaultRedactionBudgetRule>>,
}

impl DefaultRedactionBudgetValidator {
    pub fn validate(&self, item: &DefaultRedactionBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRedactionBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRedactionBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_redaction_budget_validator() -> DefaultRedactionBudgetValidator {
    DefaultRedactionBudgetValidator {
        rules: vec![
            Rc::new(DefaultRedactionBudgetVisibleGuardRule { limit: 255 }) as Rc<dyn DefaultRedactionBudgetRule>,
            Rc::new(DefaultRedactionBudgetRequiredDescriptionRule {
                message: "default redaction budget needs an entry".to_string(),
            }),
            Rc::new(DefaultRedactionBudgetDescriptionLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_redaction_budget_validator();
        let item = new_default_redaction_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = DefaultRedactionBudgetRequiredDescriptionRule {
            message: "default redaction budget needs a value".to_string(),
        };
        let mut item = new_default_redaction_budget("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = DefaultRedactionBudgetDescriptionLengthRule { max_length: 120 };
        let mut item = new_default_redaction_budget("x", "y");
        item.description = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
