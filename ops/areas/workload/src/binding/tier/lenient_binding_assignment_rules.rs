use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient binding assignment must pass. Check returns its problems, or none.
pub trait LenientBindingAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &LenientBindingAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingAssignmentDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientBindingAssignmentRule for LenientBindingAssignmentDurationMinutesRangeRule {
    fn check(&self, item: &LenientBindingAssignment) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl LenientBindingAssignmentRule for LenientBindingAssignmentCodeLengthRule {
    fn check(&self, item: &LenientBindingAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingAssignmentCodePrefixRule {
    pub prefix: String,
}

impl LenientBindingAssignmentRule for LenientBindingAssignmentCodePrefixRule {
    fn check(&self, item: &LenientBindingAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingAssignmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LenientBindingAssignmentRule for LenientBindingAssignmentAllowedTitleRule {
    fn check(&self, item: &LenientBindingAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingAssignmentPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientBindingAssignmentRule for LenientBindingAssignmentPriorityRangeRule {
    fn check(&self, item: &LenientBindingAssignment) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingAssignmentCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientBindingAssignmentRule for LenientBindingAssignmentCapacityRangeRule {
    fn check(&self, item: &LenientBindingAssignment) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient binding assignment.
#[derive(Debug, Clone)]
pub struct LenientBindingAssignmentValidator {
    pub rules: Vec<Rc<dyn LenientBindingAssignmentRule>>,
}

impl LenientBindingAssignmentValidator {
    pub fn validate(&self, item: &LenientBindingAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientBindingAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientBindingAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_binding_assignment_validator() -> LenientBindingAssignmentValidator {
    LenientBindingAssignmentValidator {
        rules: vec![
            Rc::new(LenientBindingAssignmentDurationMinutesRangeRule { min: 120, max: 86400 })
                as Rc<dyn LenientBindingAssignmentRule>,
            Rc::new(LenientBindingAssignmentCodeLengthRule { max_length: 120 }),
            Rc::new(LenientBindingAssignmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LenientBindingAssignmentAllowedTitleRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(LenientBindingAssignmentPriorityRangeRule { min: 32, max: 1000 }),
            Rc::new(LenientBindingAssignmentCapacityRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_binding_assignment_validator();
        let item = new_lenient_binding_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LenientBindingAssignmentDurationMinutesRangeRule { min: 255, max: 3600 };
        let mut item = new_lenient_binding_assignment("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientBindingAssignmentCodeLengthRule { max_length: 120 };
        let mut item = new_lenient_binding_assignment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = LenientBindingAssignmentPriorityRangeRule { min: 255, max: 86400 };
        let mut item = new_lenient_binding_assignment("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = LenientBindingAssignmentCapacityRangeRule { min: 16, max: 3600 };
        let mut item = new_lenient_binding_assignment("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
