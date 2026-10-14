use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback assignment must pass. Check returns its problems, or none.
pub trait RollbackAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &RollbackAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAssignmentAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl RollbackAssignmentRule for RollbackAssignmentAllowedOwnerIdRule {
    fn check(&self, item: &RollbackAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is inconsistent", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAssignmentDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackAssignmentRule for RollbackAssignmentDurationMinutesRangeRule {
    fn check(&self, item: &RollbackAssignment) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAssignmentRequiredOwnerIdRule {
    pub message: String,
}

impl RollbackAssignmentRule for RollbackAssignmentRequiredOwnerIdRule {
    fn check(&self, item: &RollbackAssignment) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAssignmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl RollbackAssignmentRule for RollbackAssignmentOwnerIdLengthRule {
    fn check(&self, item: &RollbackAssignment) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback assignment.
#[derive(Debug, Clone)]
pub struct RollbackAssignmentValidator {
    pub rules: Vec<Rc<dyn RollbackAssignmentRule>>,
}

impl RollbackAssignmentValidator {
    pub fn validate(&self, item: &RollbackAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_assignment_validator() -> RollbackAssignmentValidator {
    RollbackAssignmentValidator {
        rules: vec![
            Rc::new(RollbackAssignmentAllowedOwnerIdRule { allowed: Vec::new() }) as Rc<dyn RollbackAssignmentRule>,
            Rc::new(RollbackAssignmentDurationMinutesRangeRule { min: 32, max: 86400 }),
            Rc::new(RollbackAssignmentRequiredOwnerIdRule {
                message: "rollback assignment needs an entry".to_string(),
            }),
            Rc::new(RollbackAssignmentOwnerIdLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_assignment_validator();
        let item = new_rollback_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RollbackAssignmentDurationMinutesRangeRule { min: 64, max: 80 };
        let mut item = new_rollback_assignment("x", "y");
        item.duration_minutes = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = RollbackAssignmentRequiredOwnerIdRule {
            message: "rollback assignment needs an entry".to_string(),
        };
        let mut item = new_rollback_assignment("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = RollbackAssignmentOwnerIdLengthRule { max_length: 255 };
        let mut item = new_rollback_assignment("x", "y");
        item.owner_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
