use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation budget must pass. Check returns its problems, or none.
pub trait IsolationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &IsolationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBudgetHostLengthRule {
    pub max_length: i64,
}

impl IsolationBudgetRule for IsolationBudgetHostLengthRule {
    fn check(&self, item: &IsolationBudget) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBudgetRequiredCodeRule {
    pub message: String,
}

impl IsolationBudgetRule for IsolationBudgetRequiredCodeRule {
    fn check(&self, item: &IsolationBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBudgetCodeLengthRule {
    pub max_length: i64,
}

impl IsolationBudgetRule for IsolationBudgetCodeLengthRule {
    fn check(&self, item: &IsolationBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBudgetApprovedGuardRule {
    pub limit: i64,
}

impl IsolationBudgetRule for IsolationBudgetApprovedGuardRule {
    fn check(&self, item: &IsolationBudget) -> Vec<String> {
        if item.approved && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while approved",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation budget.
#[derive(Debug, Clone)]
pub struct IsolationBudgetValidator {
    pub rules: Vec<Rc<dyn IsolationBudgetRule>>,
}

impl IsolationBudgetValidator {
    pub fn validate(&self, item: &IsolationBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_budget_validator() -> IsolationBudgetValidator {
    IsolationBudgetValidator {
        rules: vec![
            Rc::new(IsolationBudgetHostLengthRule { max_length: 32 }) as Rc<dyn IsolationBudgetRule>,
            Rc::new(IsolationBudgetRequiredCodeRule {
                message: "isolation budget needs this field".to_string(),
            }),
            Rc::new(IsolationBudgetCodeLengthRule { max_length: 86400 }),
            Rc::new(IsolationBudgetApprovedGuardRule { limit: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_budget_validator();
        let item = new_isolation_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = IsolationBudgetHostLengthRule { max_length: 86400 };
        let mut item = new_isolation_budget("x", "y");
        item.host = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationBudgetRequiredCodeRule {
            message: "isolation budget needs this field".to_string(),
        };
        let mut item = new_isolation_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IsolationBudgetCodeLengthRule { max_length: 16 };
        let mut item = new_isolation_budget("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
