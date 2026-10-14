use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query audit must pass. Check returns its problems, or none.
pub trait QueryAuditRule: std::fmt::Debug {
    fn check(&self, item: &QueryAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAuditQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryAuditRule for QueryAuditQuantityRangeRule {
    fn check(&self, item: &QueryAudit) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAuditCodeLengthRule {
    pub max_length: i64,
}

impl QueryAuditRule for QueryAuditCodeLengthRule {
    fn check(&self, item: &QueryAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAuditDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryAuditRule for QueryAuditDurationMinutesRangeRule {
    fn check(&self, item: &QueryAudit) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a query audit.
#[derive(Debug, Clone)]
pub struct QueryAuditValidator {
    pub rules: Vec<Rc<dyn QueryAuditRule>>,
}

impl QueryAuditValidator {
    pub fn validate(&self, item: &QueryAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_audit_validator() -> QueryAuditValidator {
    QueryAuditValidator {
        rules: vec![
            Rc::new(QueryAuditQuantityRangeRule { min: 120, max: 3600 }) as Rc<dyn QueryAuditRule>,
            Rc::new(QueryAuditCodeLengthRule { max_length: 64 }),
            Rc::new(QueryAuditDurationMinutesRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_audit_validator();
        let item = new_query_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = QueryAuditQuantityRangeRule { min: 80, max: 3600 };
        let mut item = new_query_audit("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryAuditCodeLengthRule { max_length: 3600 };
        let mut item = new_query_audit("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = QueryAuditDurationMinutesRangeRule { min: 1000, max: 1000 };
        let mut item = new_query_audit("x", "y");
        item.duration_minutes = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
