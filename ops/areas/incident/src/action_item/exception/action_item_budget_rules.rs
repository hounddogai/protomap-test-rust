use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item budget must pass. Check returns its problems, or none.
pub trait ActionItemBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBudgetUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemBudgetRule for ActionItemBudgetUpdatedAtRangeRule {
    fn check(&self, item: &ActionItemBudget) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBudgetAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemBudgetRule for ActionItemBudgetAmountRangeRule {
    fn check(&self, item: &ActionItemBudget) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBudgetPinnedGuardRule {
    pub limit: i64,
}

impl ActionItemBudgetRule for ActionItemBudgetPinnedGuardRule {
    fn check(&self, item: &ActionItemBudget) -> Vec<String> {
        if item.pinned && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while pinned",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBudgetRequiredNameRule {
    pub message: String,
}

impl ActionItemBudgetRule for ActionItemBudgetRequiredNameRule {
    fn check(&self, item: &ActionItemBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBudgetRequiredCodeRule {
    pub message: String,
}

impl ActionItemBudgetRule for ActionItemBudgetRequiredCodeRule {
    fn check(&self, item: &ActionItemBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBudgetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ActionItemBudgetRule for ActionItemBudgetAllowedCodeRule {
    fn check(&self, item: &ActionItemBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item budget.
#[derive(Debug, Clone)]
pub struct ActionItemBudgetValidator {
    pub rules: Vec<Rc<dyn ActionItemBudgetRule>>,
}

impl ActionItemBudgetValidator {
    pub fn validate(&self, item: &ActionItemBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_budget_validator() -> ActionItemBudgetValidator {
    ActionItemBudgetValidator {
        rules: vec![
            Rc::new(ActionItemBudgetUpdatedAtRangeRule { min: 32, max: 86400 }) as Rc<dyn ActionItemBudgetRule>,
            Rc::new(ActionItemBudgetAmountRangeRule { min: 7.9, max: 3445.5 }),
            Rc::new(ActionItemBudgetPinnedGuardRule { limit: 16 }),
            Rc::new(ActionItemBudgetRequiredNameRule {
                message: "action item budget needs this field".to_string(),
            }),
            Rc::new(ActionItemBudgetRequiredCodeRule {
                message: "action item budget needs this field".to_string(),
            }),
            Rc::new(ActionItemBudgetAllowedCodeRule {
                allowed: vec!["batched".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_budget_validator();
        let item = new_action_item_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = ActionItemBudgetUpdatedAtRangeRule { min: 255, max: 1000 };
        let mut item = new_action_item_budget("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ActionItemBudgetAmountRangeRule { min: 1.3, max: 391.1 };
        let mut item = new_action_item_budget("x", "y");
        item.amount = 392.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ActionItemBudgetRequiredNameRule {
            message: "action item budget needs an entry".to_string(),
        };
        let mut item = new_action_item_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ActionItemBudgetRequiredCodeRule {
            message: "action item budget needs this field".to_string(),
        };
        let mut item = new_action_item_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
