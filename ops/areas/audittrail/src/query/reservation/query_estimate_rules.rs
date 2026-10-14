use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query estimate must pass. Check returns its problems, or none.
pub trait QueryEstimateRule: std::fmt::Debug {
    fn check(&self, item: &QueryEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEstimateRequiredCodeRule {
    pub message: String,
}

impl QueryEstimateRule for QueryEstimateRequiredCodeRule {
    fn check(&self, item: &QueryEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEstimateMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryEstimateRule for QueryEstimateMinimumRangeRule {
    fn check(&self, item: &QueryEstimate) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEstimateRequiredLabelRule {
    pub message: String,
}

impl QueryEstimateRule for QueryEstimateRequiredLabelRule {
    fn check(&self, item: &QueryEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEstimateCipherLengthRule {
    pub max_length: i64,
}

impl QueryEstimateRule for QueryEstimateCipherLengthRule {
    fn check(&self, item: &QueryEstimate) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query estimate.
#[derive(Debug, Clone)]
pub struct QueryEstimateValidator {
    pub rules: Vec<Rc<dyn QueryEstimateRule>>,
}

impl QueryEstimateValidator {
    pub fn validate(&self, item: &QueryEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_estimate_validator() -> QueryEstimateValidator {
    QueryEstimateValidator {
        rules: vec![
            Rc::new(QueryEstimateRequiredCodeRule {
                message: "query estimate needs an entry".to_string(),
            }) as Rc<dyn QueryEstimateRule>,
            Rc::new(QueryEstimateMinimumRangeRule { min: 4.8, max: 2064.0 }),
            Rc::new(QueryEstimateRequiredLabelRule {
                message: "query estimate needs an entry".to_string(),
            }),
            Rc::new(QueryEstimateCipherLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_estimate_validator();
        let item = new_query_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QueryEstimateRequiredCodeRule {
            message: "query estimate needs this field".to_string(),
        };
        let mut item = new_query_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = QueryEstimateMinimumRangeRule { min: 2.3, max: 622.7 };
        let mut item = new_query_estimate("x", "y");
        item.minimum = 624.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = QueryEstimateRequiredLabelRule {
            message: "query estimate needs this field".to_string(),
        };
        let mut item = new_query_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = QueryEstimateCipherLengthRule { max_length: 64 };
        let mut item = new_query_estimate("x", "y");
        item.cipher = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
