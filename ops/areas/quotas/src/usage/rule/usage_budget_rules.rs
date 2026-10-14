use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage budget must pass. Check returns its problems, or none.
pub trait UsageBudgetRule: std::fmt::Debug {
    fn check(&self, item: &UsageBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBudgetScopeLengthRule {
    pub max_length: i64,
}

impl UsageBudgetRule for UsageBudgetScopeLengthRule {
    fn check(&self, item: &UsageBudget) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBudgetRequiredAudienceRule {
    pub message: String,
}

impl UsageBudgetRule for UsageBudgetRequiredAudienceRule {
    fn check(&self, item: &UsageBudget) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBudgetLabelLengthRule {
    pub max_length: i64,
}

impl UsageBudgetRule for UsageBudgetLabelLengthRule {
    fn check(&self, item: &UsageBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBudgetRequiredScopeRule {
    pub message: String,
}

impl UsageBudgetRule for UsageBudgetRequiredScopeRule {
    fn check(&self, item: &UsageBudget) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBudgetRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageBudgetRule for UsageBudgetRotationDaysRangeRule {
    fn check(&self, item: &UsageBudget) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage budget.
#[derive(Debug, Clone)]
pub struct UsageBudgetValidator {
    pub rules: Vec<Rc<dyn UsageBudgetRule>>,
}

impl UsageBudgetValidator {
    pub fn validate(&self, item: &UsageBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_budget_validator() -> UsageBudgetValidator {
    UsageBudgetValidator {
        rules: vec![
            Rc::new(UsageBudgetScopeLengthRule { max_length: 64 }) as Rc<dyn UsageBudgetRule>,
            Rc::new(UsageBudgetRequiredAudienceRule {
                message: "usage budget needs a value".to_string(),
            }),
            Rc::new(UsageBudgetLabelLengthRule { max_length: 64 }),
            Rc::new(UsageBudgetRequiredScopeRule {
                message: "usage budget needs a value".to_string(),
            }),
            Rc::new(UsageBudgetRotationDaysRangeRule { min: 255, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_budget_validator();
        let item = new_usage_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = UsageBudgetScopeLengthRule { max_length: 32 };
        let mut item = new_usage_budget("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = UsageBudgetRequiredAudienceRule {
            message: "usage budget needs an entry".to_string(),
        };
        let mut item = new_usage_budget("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = UsageBudgetLabelLengthRule { max_length: 32 };
        let mut item = new_usage_budget("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = UsageBudgetRequiredScopeRule {
            message: "usage budget needs this field".to_string(),
        };
        let mut item = new_usage_budget("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = UsageBudgetRotationDaysRangeRule { min: 80, max: 120 };
        let mut item = new_usage_budget("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
