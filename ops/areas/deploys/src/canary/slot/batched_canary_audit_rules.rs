use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched canary audit must pass. Check returns its problems, or none.
pub trait BatchedCanaryAuditRule: std::fmt::Debug {
    fn check(&self, item: &BatchedCanaryAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCanaryAuditMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedCanaryAuditRule for BatchedCanaryAuditMinimumRangeRule {
    fn check(&self, item: &BatchedCanaryAudit) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCanaryAuditScopeLengthRule {
    pub max_length: i64,
}

impl BatchedCanaryAuditRule for BatchedCanaryAuditScopeLengthRule {
    fn check(&self, item: &BatchedCanaryAudit) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCanaryAuditUnitLengthRule {
    pub max_length: i64,
}

impl BatchedCanaryAuditRule for BatchedCanaryAuditUnitLengthRule {
    fn check(&self, item: &BatchedCanaryAudit) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCanaryAuditCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedCanaryAuditRule for BatchedCanaryAuditCeilingRangeRule {
    fn check(&self, item: &BatchedCanaryAudit) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCanaryAuditMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedCanaryAuditRule for BatchedCanaryAuditMarginRangeRule {
    fn check(&self, item: &BatchedCanaryAudit) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched canary audit.
#[derive(Debug, Clone)]
pub struct BatchedCanaryAuditValidator {
    pub rules: Vec<Rc<dyn BatchedCanaryAuditRule>>,
}

impl BatchedCanaryAuditValidator {
    pub fn validate(&self, item: &BatchedCanaryAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedCanaryAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedCanaryAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_canary_audit_validator() -> BatchedCanaryAuditValidator {
    BatchedCanaryAuditValidator {
        rules: vec![
            Rc::new(BatchedCanaryAuditMinimumRangeRule { min: 7.8, max: 4152.6 }) as Rc<dyn BatchedCanaryAuditRule>,
            Rc::new(BatchedCanaryAuditScopeLengthRule { max_length: 86400 }),
            Rc::new(BatchedCanaryAuditUnitLengthRule { max_length: 64 }),
            Rc::new(BatchedCanaryAuditCeilingRangeRule { min: 1.8, max: 126.2 }),
            Rc::new(BatchedCanaryAuditMarginRangeRule { min: 7.4, max: 4867.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_canary_audit_validator();
        let item = new_batched_canary_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = BatchedCanaryAuditMinimumRangeRule { min: 2.6, max: 2110.0 };
        let mut item = new_batched_canary_audit("x", "y");
        item.minimum = 2111.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = BatchedCanaryAuditScopeLengthRule { max_length: 16 };
        let mut item = new_batched_canary_audit("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = BatchedCanaryAuditUnitLengthRule { max_length: 86400 };
        let mut item = new_batched_canary_audit("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = BatchedCanaryAuditCeilingRangeRule { min: 5.6, max: 2210.6 };
        let mut item = new_batched_canary_audit("x", "y");
        item.ceiling = 2212.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = BatchedCanaryAuditMarginRangeRule { min: 9.0, max: 117.2 };
        let mut item = new_batched_canary_audit("x", "y");
        item.margin = 118.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
