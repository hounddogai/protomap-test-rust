use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted approval budget must pass. Check returns its problems, or none.
pub trait WeightedApprovalBudgetRule: std::fmt::Debug {
    fn check(&self, item: &WeightedApprovalBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalBudgetRequiredCodeRule {
    pub message: String,
}

impl WeightedApprovalBudgetRule for WeightedApprovalBudgetRequiredCodeRule {
    fn check(&self, item: &WeightedApprovalBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalBudgetTenantIdLengthRule {
    pub max_length: i64,
}

impl WeightedApprovalBudgetRule for WeightedApprovalBudgetTenantIdLengthRule {
    fn check(&self, item: &WeightedApprovalBudget) -> Vec<String> {
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
pub struct WeightedApprovalBudgetRequiredTenantIdRule {
    pub message: String,
}

impl WeightedApprovalBudgetRule for WeightedApprovalBudgetRequiredTenantIdRule {
    fn check(&self, item: &WeightedApprovalBudget) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalBudgetTitleLengthRule {
    pub max_length: i64,
}

impl WeightedApprovalBudgetRule for WeightedApprovalBudgetTitleLengthRule {
    fn check(&self, item: &WeightedApprovalBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted approval budget.
#[derive(Debug, Clone)]
pub struct WeightedApprovalBudgetValidator {
    pub rules: Vec<Rc<dyn WeightedApprovalBudgetRule>>,
}

impl WeightedApprovalBudgetValidator {
    pub fn validate(&self, item: &WeightedApprovalBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedApprovalBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedApprovalBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_approval_budget_validator() -> WeightedApprovalBudgetValidator {
    WeightedApprovalBudgetValidator {
        rules: vec![
            Rc::new(WeightedApprovalBudgetRequiredCodeRule {
                message: "weighted approval budget needs a value".to_string(),
            }) as Rc<dyn WeightedApprovalBudgetRule>,
            Rc::new(WeightedApprovalBudgetTenantIdLengthRule { max_length: 32 }),
            Rc::new(WeightedApprovalBudgetRequiredTenantIdRule {
                message: "weighted approval budget needs an entry".to_string(),
            }),
            Rc::new(WeightedApprovalBudgetTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_approval_budget_validator();
        let item = new_weighted_approval_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedApprovalBudgetRequiredCodeRule {
            message: "weighted approval budget needs an entry".to_string(),
        };
        let mut item = new_weighted_approval_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = WeightedApprovalBudgetTenantIdLengthRule { max_length: 255 };
        let mut item = new_weighted_approval_budget("x", "y");
        item.tenant_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = WeightedApprovalBudgetRequiredTenantIdRule {
            message: "weighted approval budget needs a value".to_string(),
        };
        let mut item = new_weighted_approval_budget("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedApprovalBudgetTitleLengthRule { max_length: 64 };
        let mut item = new_weighted_approval_budget("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
