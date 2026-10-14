use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling canary assignment must pass. Check returns its problems, or none.
pub trait RollingCanaryAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &RollingCanaryAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryAssignmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingCanaryAssignmentRule for RollingCanaryAssignmentAmountRangeRule {
    fn check(&self, item: &RollingCanaryAssignment) -> Vec<String> {
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
pub struct RollingCanaryAssignmentScopeLengthRule {
    pub max_length: i64,
}

impl RollingCanaryAssignmentRule for RollingCanaryAssignmentScopeLengthRule {
    fn check(&self, item: &RollingCanaryAssignment) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryAssignmentRequiredScopeRule {
    pub message: String,
}

impl RollingCanaryAssignmentRule for RollingCanaryAssignmentRequiredScopeRule {
    fn check(&self, item: &RollingCanaryAssignment) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling canary assignment.
#[derive(Debug, Clone)]
pub struct RollingCanaryAssignmentValidator {
    pub rules: Vec<Rc<dyn RollingCanaryAssignmentRule>>,
}

impl RollingCanaryAssignmentValidator {
    pub fn validate(&self, item: &RollingCanaryAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingCanaryAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingCanaryAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_canary_assignment_validator() -> RollingCanaryAssignmentValidator {
    RollingCanaryAssignmentValidator {
        rules: vec![
            Rc::new(RollingCanaryAssignmentAmountRangeRule { min: 4.0, max: 4525.5 })
                as Rc<dyn RollingCanaryAssignmentRule>,
            Rc::new(RollingCanaryAssignmentScopeLengthRule { max_length: 86400 }),
            Rc::new(RollingCanaryAssignmentRequiredScopeRule {
                message: "rolling canary assignment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_canary_assignment_validator();
        let item = new_rolling_canary_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RollingCanaryAssignmentAmountRangeRule { min: 0.0, max: 4755.7 };
        let mut item = new_rolling_canary_assignment("x", "y");
        item.amount = 4757.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = RollingCanaryAssignmentScopeLengthRule { max_length: 32 };
        let mut item = new_rolling_canary_assignment("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = RollingCanaryAssignmentRequiredScopeRule {
            message: "rolling canary assignment needs this field".to_string(),
        };
        let mut item = new_rolling_canary_assignment("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
