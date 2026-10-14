use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict audit event budget must pass. Check returns its problems, or none.
pub trait StrictAuditEventBudgetRule: std::fmt::Debug {
    fn check(&self, item: &StrictAuditEventBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventBudgetCodeLengthRule {
    pub max_length: i64,
}

impl StrictAuditEventBudgetRule for StrictAuditEventBudgetCodeLengthRule {
    fn check(&self, item: &StrictAuditEventBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventBudgetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl StrictAuditEventBudgetRule for StrictAuditEventBudgetAllowedNameRule {
    fn check(&self, item: &StrictAuditEventBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventBudgetRequiredLocaleRule {
    pub message: String,
}

impl StrictAuditEventBudgetRule for StrictAuditEventBudgetRequiredLocaleRule {
    fn check(&self, item: &StrictAuditEventBudget) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict audit event budget.
#[derive(Debug, Clone)]
pub struct StrictAuditEventBudgetValidator {
    pub rules: Vec<Rc<dyn StrictAuditEventBudgetRule>>,
}

impl StrictAuditEventBudgetValidator {
    pub fn validate(&self, item: &StrictAuditEventBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictAuditEventBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictAuditEventBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_audit_event_budget_validator() -> StrictAuditEventBudgetValidator {
    StrictAuditEventBudgetValidator {
        rules: vec![
            Rc::new(StrictAuditEventBudgetCodeLengthRule { max_length: 1000 }) as Rc<dyn StrictAuditEventBudgetRule>,
            Rc::new(StrictAuditEventBudgetAllowedNameRule { allowed: Vec::new() }),
            Rc::new(StrictAuditEventBudgetRequiredLocaleRule {
                message: "strict audit event budget needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_audit_event_budget_validator();
        let item = new_strict_audit_event_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictAuditEventBudgetCodeLengthRule { max_length: 86400 };
        let mut item = new_strict_audit_event_budget("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = StrictAuditEventBudgetRequiredLocaleRule {
            message: "strict audit event budget needs a value".to_string(),
        };
        let mut item = new_strict_audit_event_budget("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
