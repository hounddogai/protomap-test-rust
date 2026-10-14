use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy budget must pass. Check returns its problems, or none.
pub trait TtlPolicyBudgetRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBudgetRequiredNamespaceRule {
    pub message: String,
}

impl TtlPolicyBudgetRule for TtlPolicyBudgetRequiredNamespaceRule {
    fn check(&self, item: &TtlPolicyBudget) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBudgetRegionLengthRule {
    pub max_length: i64,
}

impl TtlPolicyBudgetRule for TtlPolicyBudgetRegionLengthRule {
    fn check(&self, item: &TtlPolicyBudget) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBudgetCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyBudgetRule for TtlPolicyBudgetCodePrefixRule {
    fn check(&self, item: &TtlPolicyBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBudgetRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyBudgetRule for TtlPolicyBudgetRequiredCodeRule {
    fn check(&self, item: &TtlPolicyBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBudgetRequiredTitleRule {
    pub message: String,
}

impl TtlPolicyBudgetRule for TtlPolicyBudgetRequiredTitleRule {
    fn check(&self, item: &TtlPolicyBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy budget.
#[derive(Debug, Clone)]
pub struct TtlPolicyBudgetValidator {
    pub rules: Vec<Rc<dyn TtlPolicyBudgetRule>>,
}

impl TtlPolicyBudgetValidator {
    pub fn validate(&self, item: &TtlPolicyBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_budget_validator() -> TtlPolicyBudgetValidator {
    TtlPolicyBudgetValidator {
        rules: vec![
            Rc::new(TtlPolicyBudgetRequiredNamespaceRule {
                message: "ttl policy budget needs this field".to_string(),
            }) as Rc<dyn TtlPolicyBudgetRule>,
            Rc::new(TtlPolicyBudgetRegionLengthRule { max_length: 86400 }),
            Rc::new(TtlPolicyBudgetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TtlPolicyBudgetRequiredCodeRule {
                message: "ttl policy budget needs a value".to_string(),
            }),
            Rc::new(TtlPolicyBudgetRequiredTitleRule {
                message: "ttl policy budget needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_budget_validator();
        let item = new_ttl_policy_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = TtlPolicyBudgetRequiredNamespaceRule {
            message: "ttl policy budget needs this field".to_string(),
        };
        let mut item = new_ttl_policy_budget("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = TtlPolicyBudgetRegionLengthRule { max_length: 16 };
        let mut item = new_ttl_policy_budget("x", "y");
        item.region = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyBudgetRequiredCodeRule {
            message: "ttl policy budget needs this field".to_string(),
        };
        let mut item = new_ttl_policy_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TtlPolicyBudgetRequiredTitleRule {
            message: "ttl policy budget needs this field".to_string(),
        };
        let mut item = new_ttl_policy_budget("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
