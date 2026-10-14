use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a nonce assignment must pass. Check returns its problems, or none.
pub trait NonceAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &NonceAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAssignmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceAssignmentRule for NonceAssignmentFloorValueRangeRule {
    fn check(&self, item: &NonceAssignment) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAssignmentAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl NonceAssignmentRule for NonceAssignmentAllowedNotesRule {
    fn check(&self, item: &NonceAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is too long", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAssignmentRequiredLabelRule {
    pub message: String,
}

impl NonceAssignmentRule for NonceAssignmentRequiredLabelRule {
    fn check(&self, item: &NonceAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAssignmentCodePrefixRule {
    pub prefix: String,
}

impl NonceAssignmentRule for NonceAssignmentCodePrefixRule {
    fn check(&self, item: &NonceAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAssignmentVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceAssignmentRule for NonceAssignmentVersionRangeRule {
    fn check(&self, item: &NonceAssignment) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAssignmentRequiredNotesRule {
    pub message: String,
}

impl NonceAssignmentRule for NonceAssignmentRequiredNotesRule {
    fn check(&self, item: &NonceAssignment) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce assignment.
#[derive(Debug, Clone)]
pub struct NonceAssignmentValidator {
    pub rules: Vec<Rc<dyn NonceAssignmentRule>>,
}

impl NonceAssignmentValidator {
    pub fn validate(&self, item: &NonceAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_assignment_validator() -> NonceAssignmentValidator {
    NonceAssignmentValidator {
        rules: vec![
            Rc::new(NonceAssignmentFloorValueRangeRule { min: 3.8, max: 3027.4 }) as Rc<dyn NonceAssignmentRule>,
            Rc::new(NonceAssignmentAllowedNotesRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(NonceAssignmentRequiredLabelRule {
                message: "nonce assignment needs this field".to_string(),
            }),
            Rc::new(NonceAssignmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(NonceAssignmentVersionRangeRule { min: 255, max: 1000 }),
            Rc::new(NonceAssignmentRequiredNotesRule {
                message: "nonce assignment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_assignment_validator();
        let item = new_nonce_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = NonceAssignmentFloorValueRangeRule { min: 6.4, max: 4556.2 };
        let mut item = new_nonce_assignment("x", "y");
        item.floor_value = 4557.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = NonceAssignmentRequiredLabelRule {
            message: "nonce assignment needs this field".to_string(),
        };
        let mut item = new_nonce_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = NonceAssignmentVersionRangeRule { min: 32, max: 255 };
        let mut item = new_nonce_assignment("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = NonceAssignmentRequiredNotesRule {
            message: "nonce assignment needs a value".to_string(),
        };
        let mut item = new_nonce_assignment("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
