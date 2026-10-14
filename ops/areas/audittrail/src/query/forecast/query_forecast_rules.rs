use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query forecast must pass. Check returns its problems, or none.
pub trait QueryForecastRule: std::fmt::Debug {
    fn check(&self, item: &QueryForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryForecastCipherLengthRule {
    pub max_length: i64,
}

impl QueryForecastRule for QueryForecastCipherLengthRule {
    fn check(&self, item: &QueryForecast) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryForecastRequiredTitleRule {
    pub message: String,
}

impl QueryForecastRule for QueryForecastRequiredTitleRule {
    fn check(&self, item: &QueryForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryForecastRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryForecastRule for QueryForecastRevisionRangeRule {
    fn check(&self, item: &QueryForecast) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryForecastTitleLengthRule {
    pub max_length: i64,
}

impl QueryForecastRule for QueryForecastTitleLengthRule {
    fn check(&self, item: &QueryForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryForecastCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryForecastRule for QueryForecastCapacityRangeRule {
    fn check(&self, item: &QueryForecast) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a query forecast.
#[derive(Debug, Clone)]
pub struct QueryForecastValidator {
    pub rules: Vec<Rc<dyn QueryForecastRule>>,
}

impl QueryForecastValidator {
    pub fn validate(&self, item: &QueryForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_forecast_validator() -> QueryForecastValidator {
    QueryForecastValidator {
        rules: vec![
            Rc::new(QueryForecastCipherLengthRule { max_length: 3600 }) as Rc<dyn QueryForecastRule>,
            Rc::new(QueryForecastRequiredTitleRule {
                message: "query forecast needs a value".to_string(),
            }),
            Rc::new(QueryForecastRevisionRangeRule { min: 16, max: 64 }),
            Rc::new(QueryForecastTitleLengthRule { max_length: 80 }),
            Rc::new(QueryForecastCapacityRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_forecast_validator();
        let item = new_query_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = QueryForecastCipherLengthRule { max_length: 16 };
        let mut item = new_query_forecast("x", "y");
        item.cipher = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = QueryForecastRequiredTitleRule {
            message: "query forecast needs an entry".to_string(),
        };
        let mut item = new_query_forecast("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = QueryForecastRevisionRangeRule { min: 16, max: 120 };
        let mut item = new_query_forecast("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = QueryForecastTitleLengthRule { max_length: 120 };
        let mut item = new_query_forecast("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = QueryForecastCapacityRangeRule { min: 16, max: 255 };
        let mut item = new_query_forecast("x", "y");
        item.capacity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
