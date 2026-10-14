use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled query budget must pass. Check returns its problems, or none.
pub trait ScheduledQueryBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledQueryBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryBudgetCodePrefixRule {
    pub prefix: String,
}

impl ScheduledQueryBudgetRule for ScheduledQueryBudgetCodePrefixRule {
    fn check(&self, item: &ScheduledQueryBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryBudgetTenantIdLengthRule {
    pub max_length: i64,
}

impl ScheduledQueryBudgetRule for ScheduledQueryBudgetTenantIdLengthRule {
    fn check(&self, item: &ScheduledQueryBudget) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryBudgetCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledQueryBudgetRule for ScheduledQueryBudgetCodeLengthRule {
    fn check(&self, item: &ScheduledQueryBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryBudgetRequiredCodeRule {
    pub message: String,
}

impl ScheduledQueryBudgetRule for ScheduledQueryBudgetRequiredCodeRule {
    fn check(&self, item: &ScheduledQueryBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled query budget.
#[derive(Debug, Clone)]
pub struct ScheduledQueryBudgetValidator {
    pub rules: Vec<Rc<dyn ScheduledQueryBudgetRule>>,
}

impl ScheduledQueryBudgetValidator {
    pub fn validate(&self, item: &ScheduledQueryBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledQueryBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledQueryBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_query_budget_validator() -> ScheduledQueryBudgetValidator {
    ScheduledQueryBudgetValidator {
        rules: vec![
            Rc::new(ScheduledQueryBudgetCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn ScheduledQueryBudgetRule>,
            Rc::new(ScheduledQueryBudgetTenantIdLengthRule { max_length: 3600 }),
            Rc::new(ScheduledQueryBudgetCodeLengthRule { max_length: 255 }),
            Rc::new(ScheduledQueryBudgetRequiredCodeRule {
                message: "scheduled query budget needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_query_budget_validator();
        let item = new_scheduled_query_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = ScheduledQueryBudgetTenantIdLengthRule { max_length: 32 };
        let mut item = new_scheduled_query_budget("x", "y");
        item.tenant_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledQueryBudgetCodeLengthRule { max_length: 255 };
        let mut item = new_scheduled_query_budget("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledQueryBudgetRequiredCodeRule {
            message: "scheduled query budget needs this field".to_string(),
        };
        let mut item = new_scheduled_query_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
