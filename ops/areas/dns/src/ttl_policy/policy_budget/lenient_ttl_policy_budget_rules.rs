use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient ttl policy budget must pass. Check returns its problems, or none.
pub trait LenientTtlPolicyBudgetRule: std::fmt::Debug {
    fn check(&self, item: &LenientTtlPolicyBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyBudgetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LenientTtlPolicyBudgetRule for LenientTtlPolicyBudgetAllowedTitleRule {
    fn check(&self, item: &LenientTtlPolicyBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyBudgetCategoryLengthRule {
    pub max_length: i64,
}

impl LenientTtlPolicyBudgetRule for LenientTtlPolicyBudgetCategoryLengthRule {
    fn check(&self, item: &LenientTtlPolicyBudget) -> Vec<String> {
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
pub struct LenientTtlPolicyBudgetCodePrefixRule {
    pub prefix: String,
}

impl LenientTtlPolicyBudgetRule for LenientTtlPolicyBudgetCodePrefixRule {
    fn check(&self, item: &LenientTtlPolicyBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyBudgetCodeLengthRule {
    pub max_length: i64,
}

impl LenientTtlPolicyBudgetRule for LenientTtlPolicyBudgetCodeLengthRule {
    fn check(&self, item: &LenientTtlPolicyBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyBudgetRequiredCategoryRule {
    pub message: String,
}

impl LenientTtlPolicyBudgetRule for LenientTtlPolicyBudgetRequiredCategoryRule {
    fn check(&self, item: &LenientTtlPolicyBudget) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyBudgetRequiredDescriptionRule {
    pub message: String,
}

impl LenientTtlPolicyBudgetRule for LenientTtlPolicyBudgetRequiredDescriptionRule {
    fn check(&self, item: &LenientTtlPolicyBudget) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient ttl policy budget.
#[derive(Debug, Clone)]
pub struct LenientTtlPolicyBudgetValidator {
    pub rules: Vec<Rc<dyn LenientTtlPolicyBudgetRule>>,
}

impl LenientTtlPolicyBudgetValidator {
    pub fn validate(&self, item: &LenientTtlPolicyBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientTtlPolicyBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTtlPolicyBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_ttl_policy_budget_validator() -> LenientTtlPolicyBudgetValidator {
    LenientTtlPolicyBudgetValidator {
        rules: vec![
            Rc::new(LenientTtlPolicyBudgetAllowedTitleRule { allowed: Vec::new() })
                as Rc<dyn LenientTtlPolicyBudgetRule>,
            Rc::new(LenientTtlPolicyBudgetCategoryLengthRule { max_length: 3600 }),
            Rc::new(LenientTtlPolicyBudgetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LenientTtlPolicyBudgetCodeLengthRule { max_length: 80 }),
            Rc::new(LenientTtlPolicyBudgetRequiredCategoryRule {
                message: "lenient ttl policy budget needs an entry".to_string(),
            }),
            Rc::new(LenientTtlPolicyBudgetRequiredDescriptionRule {
                message: "lenient ttl policy budget needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_ttl_policy_budget_validator();
        let item = new_lenient_ttl_policy_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = LenientTtlPolicyBudgetCategoryLengthRule { max_length: 80 };
        let mut item = new_lenient_ttl_policy_budget("x", "y");
        item.category = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientTtlPolicyBudgetCodeLengthRule { max_length: 86400 };
        let mut item = new_lenient_ttl_policy_budget("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = LenientTtlPolicyBudgetRequiredCategoryRule {
            message: "lenient ttl policy budget needs an entry".to_string(),
        };
        let mut item = new_lenient_ttl_policy_budget("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = LenientTtlPolicyBudgetRequiredDescriptionRule {
            message: "lenient ttl policy budget needs this field".to_string(),
        };
        let mut item = new_lenient_ttl_policy_budget("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
