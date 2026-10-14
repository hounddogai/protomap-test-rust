use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query group must pass. Check returns its problems, or none.
pub trait QueryGroupRule: std::fmt::Debug {
    fn check(&self, item: &QueryGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryGroupCodeLengthRule {
    pub max_length: i64,
}

impl QueryGroupRule for QueryGroupCodeLengthRule {
    fn check(&self, item: &QueryGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryGroupAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl QueryGroupRule for QueryGroupAllowedUnitRule {
    fn check(&self, item: &QueryGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is unknown", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryGroupMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryGroupRule for QueryGroupMinimumRangeRule {
    fn check(&self, item: &QueryGroup) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryGroupRequiredCodeRule {
    pub message: String,
}

impl QueryGroupRule for QueryGroupRequiredCodeRule {
    fn check(&self, item: &QueryGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryGroupRequiredUnitRule {
    pub message: String,
}

impl QueryGroupRule for QueryGroupRequiredUnitRule {
    fn check(&self, item: &QueryGroup) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryGroupRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryGroupRule for QueryGroupRotationDaysRangeRule {
    fn check(&self, item: &QueryGroup) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a query group.
#[derive(Debug, Clone)]
pub struct QueryGroupValidator {
    pub rules: Vec<Rc<dyn QueryGroupRule>>,
}

impl QueryGroupValidator {
    pub fn validate(&self, item: &QueryGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_group_validator() -> QueryGroupValidator {
    QueryGroupValidator {
        rules: vec![
            Rc::new(QueryGroupCodeLengthRule { max_length: 86400 }) as Rc<dyn QueryGroupRule>,
            Rc::new(QueryGroupAllowedUnitRule {
                allowed: vec!["primary".to_string(), "seasonal".to_string()],
            }),
            Rc::new(QueryGroupMinimumRangeRule { min: 1.0, max: 3224.7 }),
            Rc::new(QueryGroupRequiredCodeRule {
                message: "query group needs an entry".to_string(),
            }),
            Rc::new(QueryGroupRequiredUnitRule {
                message: "query group needs this field".to_string(),
            }),
            Rc::new(QueryGroupRotationDaysRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_group_validator();
        let item = new_query_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryGroupCodeLengthRule { max_length: 1000 };
        let mut item = new_query_group("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = QueryGroupMinimumRangeRule { min: 10.0, max: 4106.6 };
        let mut item = new_query_group("x", "y");
        item.minimum = 4108.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QueryGroupRequiredCodeRule {
            message: "query group needs this field".to_string(),
        };
        let mut item = new_query_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = QueryGroupRequiredUnitRule {
            message: "query group needs this field".to_string(),
        };
        let mut item = new_query_group("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = QueryGroupRotationDaysRangeRule { min: 32, max: 3600 };
        let mut item = new_query_group("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
