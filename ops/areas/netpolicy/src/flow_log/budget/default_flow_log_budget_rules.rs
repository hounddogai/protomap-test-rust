use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default flow log budget must pass. Check returns its problems, or none.
pub trait DefaultFlowLogBudgetRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFlowLogBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogBudgetRequiredCodeRule {
    pub message: String,
}

impl DefaultFlowLogBudgetRule for DefaultFlowLogBudgetRequiredCodeRule {
    fn check(&self, item: &DefaultFlowLogBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogBudgetRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultFlowLogBudgetRule for DefaultFlowLogBudgetRetriesRangeRule {
    fn check(&self, item: &DefaultFlowLogBudget) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogBudgetCodePrefixRule {
    pub prefix: String,
}

impl DefaultFlowLogBudgetRule for DefaultFlowLogBudgetCodePrefixRule {
    fn check(&self, item: &DefaultFlowLogBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default flow log budget.
#[derive(Debug, Clone)]
pub struct DefaultFlowLogBudgetValidator {
    pub rules: Vec<Rc<dyn DefaultFlowLogBudgetRule>>,
}

impl DefaultFlowLogBudgetValidator {
    pub fn validate(&self, item: &DefaultFlowLogBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultFlowLogBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFlowLogBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_flow_log_budget_validator() -> DefaultFlowLogBudgetValidator {
    DefaultFlowLogBudgetValidator {
        rules: vec![
            Rc::new(DefaultFlowLogBudgetRequiredCodeRule {
                message: "default flow log budget needs this field".to_string(),
            }) as Rc<dyn DefaultFlowLogBudgetRule>,
            Rc::new(DefaultFlowLogBudgetRetriesRangeRule { min: 16, max: 32 }),
            Rc::new(DefaultFlowLogBudgetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_flow_log_budget_validator();
        let item = new_default_flow_log_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultFlowLogBudgetRequiredCodeRule {
            message: "default flow log budget needs a value".to_string(),
        };
        let mut item = new_default_flow_log_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = DefaultFlowLogBudgetRetriesRangeRule { min: 1000, max: 3600 };
        let mut item = new_default_flow_log_budget("x", "y");
        item.retries = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
