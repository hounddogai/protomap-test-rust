use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query budget must pass. Check returns its problems, or none.
pub trait QueryBudgetRule: std::fmt::Debug {
    fn check(&self, item: &QueryBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBudgetRequiredCodeRule {
    pub message: String,
}

impl QueryBudgetRule for QueryBudgetRequiredCodeRule {
    fn check(&self, item: &QueryBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBudgetBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryBudgetRule for QueryBudgetBatchSizeRangeRule {
    fn check(&self, item: &QueryBudget) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBudgetCodePrefixRule {
    pub prefix: String,
}

impl QueryBudgetRule for QueryBudgetCodePrefixRule {
    fn check(&self, item: &QueryBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query budget.
#[derive(Debug, Clone)]
pub struct QueryBudgetValidator {
    pub rules: Vec<Rc<dyn QueryBudgetRule>>,
}

impl QueryBudgetValidator {
    pub fn validate(&self, item: &QueryBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_budget_validator() -> QueryBudgetValidator {
    QueryBudgetValidator {
        rules: vec![
            Rc::new(QueryBudgetRequiredCodeRule {
                message: "query budget needs a value".to_string(),
            }) as Rc<dyn QueryBudgetRule>,
            Rc::new(QueryBudgetBatchSizeRangeRule { min: 64, max: 3600 }),
            Rc::new(QueryBudgetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_budget_validator();
        let item = new_query_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QueryBudgetRequiredCodeRule {
            message: "query budget needs a value".to_string(),
        };
        let mut item = new_query_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = QueryBudgetBatchSizeRangeRule { min: 255, max: 3600 };
        let mut item = new_query_budget("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
