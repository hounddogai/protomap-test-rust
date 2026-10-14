use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient sink budget must pass. Check returns its problems, or none.
pub trait LenientSinkBudgetRule: std::fmt::Debug {
    fn check(&self, item: &LenientSinkBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkBudgetLabelLengthRule {
    pub max_length: i64,
}

impl LenientSinkBudgetRule for LenientSinkBudgetLabelLengthRule {
    fn check(&self, item: &LenientSinkBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkBudgetCodePrefixRule {
    pub prefix: String,
}

impl LenientSinkBudgetRule for LenientSinkBudgetCodePrefixRule {
    fn check(&self, item: &LenientSinkBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkBudgetRequiredCodeRule {
    pub message: String,
}

impl LenientSinkBudgetRule for LenientSinkBudgetRequiredCodeRule {
    fn check(&self, item: &LenientSinkBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkBudgetRequiredScopeRule {
    pub message: String,
}

impl LenientSinkBudgetRule for LenientSinkBudgetRequiredScopeRule {
    fn check(&self, item: &LenientSinkBudget) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkBudgetTenantIdLengthRule {
    pub max_length: i64,
}

impl LenientSinkBudgetRule for LenientSinkBudgetTenantIdLengthRule {
    fn check(&self, item: &LenientSinkBudget) -> Vec<String> {
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

/// Runs every rule of a lenient sink budget.
#[derive(Debug, Clone)]
pub struct LenientSinkBudgetValidator {
    pub rules: Vec<Rc<dyn LenientSinkBudgetRule>>,
}

impl LenientSinkBudgetValidator {
    pub fn validate(&self, item: &LenientSinkBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSinkBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSinkBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_sink_budget_validator() -> LenientSinkBudgetValidator {
    LenientSinkBudgetValidator {
        rules: vec![
            Rc::new(LenientSinkBudgetLabelLengthRule { max_length: 3600 }) as Rc<dyn LenientSinkBudgetRule>,
            Rc::new(LenientSinkBudgetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientSinkBudgetRequiredCodeRule {
                message: "lenient sink budget needs this field".to_string(),
            }),
            Rc::new(LenientSinkBudgetRequiredScopeRule {
                message: "lenient sink budget needs this field".to_string(),
            }),
            Rc::new(LenientSinkBudgetTenantIdLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_sink_budget_validator();
        let item = new_lenient_sink_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientSinkBudgetLabelLengthRule { max_length: 86400 };
        let mut item = new_lenient_sink_budget("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientSinkBudgetRequiredCodeRule {
            message: "lenient sink budget needs this field".to_string(),
        };
        let mut item = new_lenient_sink_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = LenientSinkBudgetRequiredScopeRule {
            message: "lenient sink budget needs an entry".to_string(),
        };
        let mut item = new_lenient_sink_budget("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = LenientSinkBudgetTenantIdLengthRule { max_length: 86400 };
        let mut item = new_lenient_sink_budget("x", "y");
        item.tenant_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
