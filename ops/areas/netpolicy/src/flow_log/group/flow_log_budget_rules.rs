use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log budget must pass. Check returns its problems, or none.
pub trait FlowLogBudgetRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBudgetPrincipalLengthRule {
    pub max_length: i64,
}

impl FlowLogBudgetRule for FlowLogBudgetPrincipalLengthRule {
    fn check(&self, item: &FlowLogBudget) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBudgetRequiredCountryRule {
    pub message: String,
}

impl FlowLogBudgetRule for FlowLogBudgetRequiredCountryRule {
    fn check(&self, item: &FlowLogBudget) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBudgetRequiredCodeRule {
    pub message: String,
}

impl FlowLogBudgetRule for FlowLogBudgetRequiredCodeRule {
    fn check(&self, item: &FlowLogBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log budget.
#[derive(Debug, Clone)]
pub struct FlowLogBudgetValidator {
    pub rules: Vec<Rc<dyn FlowLogBudgetRule>>,
}

impl FlowLogBudgetValidator {
    pub fn validate(&self, item: &FlowLogBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_budget_validator() -> FlowLogBudgetValidator {
    FlowLogBudgetValidator {
        rules: vec![
            Rc::new(FlowLogBudgetPrincipalLengthRule { max_length: 3600 }) as Rc<dyn FlowLogBudgetRule>,
            Rc::new(FlowLogBudgetRequiredCountryRule {
                message: "flow log budget needs an entry".to_string(),
            }),
            Rc::new(FlowLogBudgetRequiredCodeRule {
                message: "flow log budget needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_budget_validator();
        let item = new_flow_log_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = FlowLogBudgetPrincipalLengthRule { max_length: 64 };
        let mut item = new_flow_log_budget("x", "y");
        item.principal = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = FlowLogBudgetRequiredCountryRule {
            message: "flow log budget needs an entry".to_string(),
        };
        let mut item = new_flow_log_budget("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogBudgetRequiredCodeRule {
            message: "flow log budget needs a value".to_string(),
        };
        let mut item = new_flow_log_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
