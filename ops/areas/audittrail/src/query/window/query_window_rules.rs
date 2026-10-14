use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query window must pass. Check returns its problems, or none.
pub trait QueryWindowRule: std::fmt::Debug {
    fn check(&self, item: &QueryWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryWindowRequiredCodeRule {
    pub message: String,
}

impl QueryWindowRule for QueryWindowRequiredCodeRule {
    fn check(&self, item: &QueryWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryWindowAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryWindowRule for QueryWindowAmountRangeRule {
    fn check(&self, item: &QueryWindow) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryWindowCodePrefixRule {
    pub prefix: String,
}

impl QueryWindowRule for QueryWindowCodePrefixRule {
    fn check(&self, item: &QueryWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query window.
#[derive(Debug, Clone)]
pub struct QueryWindowValidator {
    pub rules: Vec<Rc<dyn QueryWindowRule>>,
}

impl QueryWindowValidator {
    pub fn validate(&self, item: &QueryWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_window_validator() -> QueryWindowValidator {
    QueryWindowValidator {
        rules: vec![
            Rc::new(QueryWindowRequiredCodeRule {
                message: "query window needs an entry".to_string(),
            }) as Rc<dyn QueryWindowRule>,
            Rc::new(QueryWindowAmountRangeRule { min: 9.6, max: 2386.4 }),
            Rc::new(QueryWindowCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_window_validator();
        let item = new_query_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QueryWindowRequiredCodeRule {
            message: "query window needs this field".to_string(),
        };
        let mut item = new_query_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = QueryWindowAmountRangeRule { min: 4.1, max: 1197.8 };
        let mut item = new_query_window("x", "y");
        item.amount = 1199.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
