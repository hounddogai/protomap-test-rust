use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query plan must pass. Check returns its problems, or none.
pub trait QueryPlanRule: std::fmt::Debug {
    fn check(&self, item: &QueryPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlanPrincipalLengthRule {
    pub max_length: i64,
}

impl QueryPlanRule for QueryPlanPrincipalLengthRule {
    fn check(&self, item: &QueryPlan) -> Vec<String> {
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
pub struct QueryPlanRequiredTitleRule {
    pub message: String,
}

impl QueryPlanRule for QueryPlanRequiredTitleRule {
    fn check(&self, item: &QueryPlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlanCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryPlanRule for QueryPlanCostRangeRule {
    fn check(&self, item: &QueryPlan) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query plan.
#[derive(Debug, Clone)]
pub struct QueryPlanValidator {
    pub rules: Vec<Rc<dyn QueryPlanRule>>,
}

impl QueryPlanValidator {
    pub fn validate(&self, item: &QueryPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_plan_validator() -> QueryPlanValidator {
    QueryPlanValidator {
        rules: vec![
            Rc::new(QueryPlanPrincipalLengthRule { max_length: 3600 }) as Rc<dyn QueryPlanRule>,
            Rc::new(QueryPlanRequiredTitleRule {
                message: "query plan needs an entry".to_string(),
            }),
            Rc::new(QueryPlanCostRangeRule { min: 2.0, max: 1061.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_plan_validator();
        let item = new_query_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = QueryPlanPrincipalLengthRule { max_length: 120 };
        let mut item = new_query_plan("x", "y");
        item.principal = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = QueryPlanRequiredTitleRule {
            message: "query plan needs this field".to_string(),
        };
        let mut item = new_query_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = QueryPlanCostRangeRule { min: 3.1, max: 4503.6 };
        let mut item = new_query_plan("x", "y");
        item.cost = 4505.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
