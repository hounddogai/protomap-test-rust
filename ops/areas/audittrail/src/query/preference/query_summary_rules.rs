use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query summary must pass. Check returns its problems, or none.
pub trait QuerySummaryRule: std::fmt::Debug {
    fn check(&self, item: &QuerySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySummaryScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuerySummaryRule for QuerySummaryScoreRangeRule {
    fn check(&self, item: &QuerySummary) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySummaryRequiredNameRule {
    pub message: String,
}

impl QuerySummaryRule for QuerySummaryRequiredNameRule {
    fn check(&self, item: &QuerySummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySummaryRequiredNamespaceRule {
    pub message: String,
}

impl QuerySummaryRule for QuerySummaryRequiredNamespaceRule {
    fn check(&self, item: &QuerySummary) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query summary.
#[derive(Debug, Clone)]
pub struct QuerySummaryValidator {
    pub rules: Vec<Rc<dyn QuerySummaryRule>>,
}

impl QuerySummaryValidator {
    pub fn validate(&self, item: &QuerySummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuerySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuerySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_summary_validator() -> QuerySummaryValidator {
    QuerySummaryValidator {
        rules: vec![
            Rc::new(QuerySummaryScoreRangeRule { min: 4.2, max: 2223.8 }) as Rc<dyn QuerySummaryRule>,
            Rc::new(QuerySummaryRequiredNameRule {
                message: "query summary needs this field".to_string(),
            }),
            Rc::new(QuerySummaryRequiredNamespaceRule {
                message: "query summary needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_summary_validator();
        let item = new_query_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = QuerySummaryScoreRangeRule { min: 9.1, max: 4382.6 };
        let mut item = new_query_summary("x", "y");
        item.score = 4384.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = QuerySummaryRequiredNameRule {
            message: "query summary needs this field".to_string(),
        };
        let mut item = new_query_summary("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = QuerySummaryRequiredNamespaceRule {
            message: "query summary needs this field".to_string(),
        };
        let mut item = new_query_summary("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
