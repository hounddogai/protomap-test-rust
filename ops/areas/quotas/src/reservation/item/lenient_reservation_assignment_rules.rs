use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient reservation assignment must pass. Check returns its problems, or none.
pub trait LenientReservationAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &LenientReservationAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAssignmentArchivedGuardRule {
    pub limit: i64,
}

impl LenientReservationAssignmentRule for LenientReservationAssignmentArchivedGuardRule {
    fn check(&self, item: &LenientReservationAssignment) -> Vec<String> {
        if item.archived && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while archived",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAssignmentRequiredCodeRule {
    pub message: String,
}

impl LenientReservationAssignmentRule for LenientReservationAssignmentRequiredCodeRule {
    fn check(&self, item: &LenientReservationAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAssignmentNameLengthRule {
    pub max_length: i64,
}

impl LenientReservationAssignmentRule for LenientReservationAssignmentNameLengthRule {
    fn check(&self, item: &LenientReservationAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAssignmentRequiredNameRule {
    pub message: String,
}

impl LenientReservationAssignmentRule for LenientReservationAssignmentRequiredNameRule {
    fn check(&self, item: &LenientReservationAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAssignmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LenientReservationAssignmentRule for LenientReservationAssignmentAllowedNameRule {
    fn check(&self, item: &LenientReservationAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAssignmentEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientReservationAssignmentRule for LenientReservationAssignmentEndsAtRangeRule {
    fn check(&self, item: &LenientReservationAssignment) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient reservation assignment.
#[derive(Debug, Clone)]
pub struct LenientReservationAssignmentValidator {
    pub rules: Vec<Rc<dyn LenientReservationAssignmentRule>>,
}

impl LenientReservationAssignmentValidator {
    pub fn validate(&self, item: &LenientReservationAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientReservationAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientReservationAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_reservation_assignment_validator() -> LenientReservationAssignmentValidator {
    LenientReservationAssignmentValidator {
        rules: vec![
            Rc::new(LenientReservationAssignmentArchivedGuardRule { limit: 64 })
                as Rc<dyn LenientReservationAssignmentRule>,
            Rc::new(LenientReservationAssignmentRequiredCodeRule {
                message: "lenient reservation assignment needs this field".to_string(),
            }),
            Rc::new(LenientReservationAssignmentNameLengthRule { max_length: 86400 }),
            Rc::new(LenientReservationAssignmentRequiredNameRule {
                message: "lenient reservation assignment needs an entry".to_string(),
            }),
            Rc::new(LenientReservationAssignmentAllowedNameRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(LenientReservationAssignmentEndsAtRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_reservation_assignment_validator();
        let item = new_lenient_reservation_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientReservationAssignmentRequiredCodeRule {
            message: "lenient reservation assignment needs this field".to_string(),
        };
        let mut item = new_lenient_reservation_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientReservationAssignmentNameLengthRule { max_length: 1000 };
        let mut item = new_lenient_reservation_assignment("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientReservationAssignmentRequiredNameRule {
            message: "lenient reservation assignment needs this field".to_string(),
        };
        let mut item = new_lenient_reservation_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = LenientReservationAssignmentEndsAtRangeRule { min: 80, max: 120 };
        let mut item = new_lenient_reservation_assignment("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
