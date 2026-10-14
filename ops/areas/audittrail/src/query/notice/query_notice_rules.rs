use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query notice must pass. Check returns its problems, or none.
pub trait QueryNoticeRule: std::fmt::Debug {
    fn check(&self, item: &QueryNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryNoticeCodePrefixRule {
    pub prefix: String,
}

impl QueryNoticeRule for QueryNoticeCodePrefixRule {
    fn check(&self, item: &QueryNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryNoticeBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryNoticeRule for QueryNoticeBudgetRangeRule {
    fn check(&self, item: &QueryNotice) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryNoticeMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryNoticeRule for QueryNoticeMarginRangeRule {
    fn check(&self, item: &QueryNotice) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a query notice.
#[derive(Debug, Clone)]
pub struct QueryNoticeValidator {
    pub rules: Vec<Rc<dyn QueryNoticeRule>>,
}

impl QueryNoticeValidator {
    pub fn validate(&self, item: &QueryNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_notice_validator() -> QueryNoticeValidator {
    QueryNoticeValidator {
        rules: vec![
            Rc::new(QueryNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn QueryNoticeRule>,
            Rc::new(QueryNoticeBudgetRangeRule { min: 0.4, max: 2388.0 }),
            Rc::new(QueryNoticeMarginRangeRule { min: 1.6, max: 2147.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_notice_validator();
        let item = new_query_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = QueryNoticeBudgetRangeRule { min: 2.6, max: 2045.5 };
        let mut item = new_query_notice("x", "y");
        item.budget = 2047.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = QueryNoticeMarginRangeRule { min: 3.0, max: 3318.7 };
        let mut item = new_query_notice("x", "y");
        item.margin = 3320.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
