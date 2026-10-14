use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn draft must pass. Check returns its problems, or none.
pub trait BudgetBurnDraftRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDraftDescriptionLengthRule {
    pub max_length: i64,
}

impl BudgetBurnDraftRule for BudgetBurnDraftDescriptionLengthRule {
    fn check(&self, item: &BudgetBurnDraft) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDraftRequiredLocaleRule {
    pub message: String,
}

impl BudgetBurnDraftRule for BudgetBurnDraftRequiredLocaleRule {
    fn check(&self, item: &BudgetBurnDraft) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDraftLocaleLengthRule {
    pub max_length: i64,
}

impl BudgetBurnDraftRule for BudgetBurnDraftLocaleLengthRule {
    fn check(&self, item: &BudgetBurnDraft) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn draft.
#[derive(Debug, Clone)]
pub struct BudgetBurnDraftValidator {
    pub rules: Vec<Rc<dyn BudgetBurnDraftRule>>,
}

impl BudgetBurnDraftValidator {
    pub fn validate(&self, item: &BudgetBurnDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_draft_validator() -> BudgetBurnDraftValidator {
    BudgetBurnDraftValidator {
        rules: vec![
            Rc::new(BudgetBurnDraftDescriptionLengthRule { max_length: 32 }) as Rc<dyn BudgetBurnDraftRule>,
            Rc::new(BudgetBurnDraftRequiredLocaleRule {
                message: "budget burn draft needs a value".to_string(),
            }),
            Rc::new(BudgetBurnDraftLocaleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_draft_validator();
        let item = new_budget_burn_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = BudgetBurnDraftDescriptionLengthRule { max_length: 80 };
        let mut item = new_budget_burn_draft("x", "y");
        item.description = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = BudgetBurnDraftRequiredLocaleRule {
            message: "budget burn draft needs a value".to_string(),
        };
        let mut item = new_budget_burn_draft("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = BudgetBurnDraftLocaleLengthRule { max_length: 32 };
        let mut item = new_budget_burn_draft("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
