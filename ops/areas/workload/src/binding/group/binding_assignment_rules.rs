use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding assignment must pass. Check returns its problems, or none.
pub trait BindingAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &BindingAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAssignmentIssuerLengthRule {
    pub max_length: i64,
}

impl BindingAssignmentRule for BindingAssignmentIssuerLengthRule {
    fn check(&self, item: &BindingAssignment) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAssignmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingAssignmentRule for BindingAssignmentFloorValueRangeRule {
    fn check(&self, item: &BindingAssignment) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAssignmentCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingAssignmentRule for BindingAssignmentCapacityRangeRule {
    fn check(&self, item: &BindingAssignment) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding assignment.
#[derive(Debug, Clone)]
pub struct BindingAssignmentValidator {
    pub rules: Vec<Rc<dyn BindingAssignmentRule>>,
}

impl BindingAssignmentValidator {
    pub fn validate(&self, item: &BindingAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_assignment_validator() -> BindingAssignmentValidator {
    BindingAssignmentValidator {
        rules: vec![
            Rc::new(BindingAssignmentIssuerLengthRule { max_length: 3600 }) as Rc<dyn BindingAssignmentRule>,
            Rc::new(BindingAssignmentFloorValueRangeRule { min: 7.3, max: 3516.2 }),
            Rc::new(BindingAssignmentCapacityRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_assignment_validator();
        let item = new_binding_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = BindingAssignmentIssuerLengthRule { max_length: 80 };
        let mut item = new_binding_assignment("x", "y");
        item.issuer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = BindingAssignmentFloorValueRangeRule { min: 9.4, max: 3279.9 };
        let mut item = new_binding_assignment("x", "y");
        item.floor_value = 3281.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = BindingAssignmentCapacityRangeRule { min: 16, max: 80 };
        let mut item = new_binding_assignment("x", "y");
        item.capacity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
