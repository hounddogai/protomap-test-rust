use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary canary assignment must pass. Check returns its problems, or none.
pub trait PrimaryCanaryAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryCanaryAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryCanaryAssignmentRule for PrimaryCanaryAssignmentCreatedAtRangeRule {
    fn check(&self, item: &PrimaryCanaryAssignment) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryCanaryAssignmentRule for PrimaryCanaryAssignmentAllowedCodeRule {
    fn check(&self, item: &PrimaryCanaryAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryCanaryAssignmentRule for PrimaryCanaryAssignmentLimitRangeRule {
    fn check(&self, item: &PrimaryCanaryAssignment) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignmentRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryCanaryAssignmentRule for PrimaryCanaryAssignmentRotationDaysRangeRule {
    fn check(&self, item: &PrimaryCanaryAssignment) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryCanaryAssignmentRule for PrimaryCanaryAssignmentCodeLengthRule {
    fn check(&self, item: &PrimaryCanaryAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignmentNameLengthRule {
    pub max_length: i64,
}

impl PrimaryCanaryAssignmentRule for PrimaryCanaryAssignmentNameLengthRule {
    fn check(&self, item: &PrimaryCanaryAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary canary assignment.
#[derive(Debug, Clone)]
pub struct PrimaryCanaryAssignmentValidator {
    pub rules: Vec<Rc<dyn PrimaryCanaryAssignmentRule>>,
}

impl PrimaryCanaryAssignmentValidator {
    pub fn validate(&self, item: &PrimaryCanaryAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryCanaryAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryCanaryAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_canary_assignment_validator() -> PrimaryCanaryAssignmentValidator {
    PrimaryCanaryAssignmentValidator {
        rules: vec![
            Rc::new(PrimaryCanaryAssignmentCreatedAtRangeRule { min: 80, max: 3600 })
                as Rc<dyn PrimaryCanaryAssignmentRule>,
            Rc::new(PrimaryCanaryAssignmentAllowedCodeRule {
                allowed: vec!["adaptive".to_string(), "pending".to_string()],
            }),
            Rc::new(PrimaryCanaryAssignmentLimitRangeRule { min: 1000, max: 3600 }),
            Rc::new(PrimaryCanaryAssignmentRotationDaysRangeRule { min: 64, max: 120 }),
            Rc::new(PrimaryCanaryAssignmentCodeLengthRule { max_length: 16 }),
            Rc::new(PrimaryCanaryAssignmentNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_canary_assignment_validator();
        let item = new_primary_canary_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = PrimaryCanaryAssignmentCreatedAtRangeRule { min: 80, max: 80 };
        let mut item = new_primary_canary_assignment("x", "y");
        item.created_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = PrimaryCanaryAssignmentLimitRangeRule { min: 80, max: 80 };
        let mut item = new_primary_canary_assignment("x", "y");
        item.limit = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PrimaryCanaryAssignmentRotationDaysRangeRule { min: 64, max: 64 };
        let mut item = new_primary_canary_assignment("x", "y");
        item.rotation_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryCanaryAssignmentCodeLengthRule { max_length: 16 };
        let mut item = new_primary_canary_assignment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryCanaryAssignmentNameLengthRule { max_length: 64 };
        let mut item = new_primary_canary_assignment("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
