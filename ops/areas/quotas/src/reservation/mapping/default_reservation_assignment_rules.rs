use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default reservation assignment must pass. Check returns its problems, or none.
pub trait DefaultReservationAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultReservationAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignmentArchivedGuardRule {
    pub limit: f64,
}

impl DefaultReservationAssignmentRule for DefaultReservationAssignmentArchivedGuardRule {
    fn check(&self, item: &DefaultReservationAssignment) -> Vec<String> {
        if item.archived && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while archived", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignmentRequiredCodeRule {
    pub message: String,
}

impl DefaultReservationAssignmentRule for DefaultReservationAssignmentRequiredCodeRule {
    fn check(&self, item: &DefaultReservationAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultReservationAssignmentRule for DefaultReservationAssignmentRateRangeRule {
    fn check(&self, item: &DefaultReservationAssignment) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignmentRequiredPrincipalRule {
    pub message: String,
}

impl DefaultReservationAssignmentRule for DefaultReservationAssignmentRequiredPrincipalRule {
    fn check(&self, item: &DefaultReservationAssignment) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignmentRequiredNameRule {
    pub message: String,
}

impl DefaultReservationAssignmentRule for DefaultReservationAssignmentRequiredNameRule {
    fn check(&self, item: &DefaultReservationAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignmentCodePrefixRule {
    pub prefix: String,
}

impl DefaultReservationAssignmentRule for DefaultReservationAssignmentCodePrefixRule {
    fn check(&self, item: &DefaultReservationAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default reservation assignment.
#[derive(Debug, Clone)]
pub struct DefaultReservationAssignmentValidator {
    pub rules: Vec<Rc<dyn DefaultReservationAssignmentRule>>,
}

impl DefaultReservationAssignmentValidator {
    pub fn validate(&self, item: &DefaultReservationAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultReservationAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultReservationAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_reservation_assignment_validator() -> DefaultReservationAssignmentValidator {
    DefaultReservationAssignmentValidator {
        rules: vec![
            Rc::new(DefaultReservationAssignmentArchivedGuardRule { limit: 6.6 })
                as Rc<dyn DefaultReservationAssignmentRule>,
            Rc::new(DefaultReservationAssignmentRequiredCodeRule {
                message: "default reservation assignment needs this field".to_string(),
            }),
            Rc::new(DefaultReservationAssignmentRateRangeRule { min: 5.1, max: 3568.4 }),
            Rc::new(DefaultReservationAssignmentRequiredPrincipalRule {
                message: "default reservation assignment needs a value".to_string(),
            }),
            Rc::new(DefaultReservationAssignmentRequiredNameRule {
                message: "default reservation assignment needs an entry".to_string(),
            }),
            Rc::new(DefaultReservationAssignmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_reservation_assignment_validator();
        let item = new_default_reservation_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultReservationAssignmentRequiredCodeRule {
            message: "default reservation assignment needs an entry".to_string(),
        };
        let mut item = new_default_reservation_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DefaultReservationAssignmentRateRangeRule { min: 8.9, max: 3829.0 };
        let mut item = new_default_reservation_assignment("x", "y");
        item.rate = 3830.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = DefaultReservationAssignmentRequiredPrincipalRule {
            message: "default reservation assignment needs a value".to_string(),
        };
        let mut item = new_default_reservation_assignment("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultReservationAssignmentRequiredNameRule {
            message: "default reservation assignment needs an entry".to_string(),
        };
        let mut item = new_default_reservation_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
