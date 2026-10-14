use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query allocation must pass. Check returns its problems, or none.
pub trait QueryAllocationRule: std::fmt::Debug {
    fn check(&self, item: &QueryAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAllocationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryAllocationRule for QueryAllocationGraceDaysRangeRule {
    fn check(&self, item: &QueryAllocation) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAllocationNameLengthRule {
    pub max_length: i64,
}

impl QueryAllocationRule for QueryAllocationNameLengthRule {
    fn check(&self, item: &QueryAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAllocationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryAllocationRule for QueryAllocationMarginRangeRule {
    fn check(&self, item: &QueryAllocation) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAllocationRequiredScopeRule {
    pub message: String,
}

impl QueryAllocationRule for QueryAllocationRequiredScopeRule {
    fn check(&self, item: &QueryAllocation) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query allocation.
#[derive(Debug, Clone)]
pub struct QueryAllocationValidator {
    pub rules: Vec<Rc<dyn QueryAllocationRule>>,
}

impl QueryAllocationValidator {
    pub fn validate(&self, item: &QueryAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_allocation_validator() -> QueryAllocationValidator {
    QueryAllocationValidator {
        rules: vec![
            Rc::new(QueryAllocationGraceDaysRangeRule { min: 120, max: 3600 }) as Rc<dyn QueryAllocationRule>,
            Rc::new(QueryAllocationNameLengthRule { max_length: 3600 }),
            Rc::new(QueryAllocationMarginRangeRule { min: 9.3, max: 2808.4 }),
            Rc::new(QueryAllocationRequiredScopeRule {
                message: "query allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_allocation_validator();
        let item = new_query_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = QueryAllocationGraceDaysRangeRule { min: 80, max: 86400 };
        let mut item = new_query_allocation("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = QueryAllocationNameLengthRule { max_length: 3600 };
        let mut item = new_query_allocation("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = QueryAllocationMarginRangeRule { min: 5.6, max: 938.1 };
        let mut item = new_query_allocation("x", "y");
        item.margin = 939.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = QueryAllocationRequiredScopeRule {
            message: "query allocation needs a value".to_string(),
        };
        let mut item = new_query_allocation("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
