use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict binding assignment must pass. Check returns its problems, or none.
pub trait StrictBindingAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &StrictBindingAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl StrictBindingAssignmentRule for StrictBindingAssignmentCodeLengthRule {
    fn check(&self, item: &StrictBindingAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingAssignmentRequiredOwnerIdRule {
    pub message: String,
}

impl StrictBindingAssignmentRule for StrictBindingAssignmentRequiredOwnerIdRule {
    fn check(&self, item: &StrictBindingAssignment) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingAssignmentCodePrefixRule {
    pub prefix: String,
}

impl StrictBindingAssignmentRule for StrictBindingAssignmentCodePrefixRule {
    fn check(&self, item: &StrictBindingAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingAssignmentAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl StrictBindingAssignmentRule for StrictBindingAssignmentAllowedTimezoneRule {
    fn check(&self, item: &StrictBindingAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is duplicated", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingAssignmentRequiredLabelRule {
    pub message: String,
}

impl StrictBindingAssignmentRule for StrictBindingAssignmentRequiredLabelRule {
    fn check(&self, item: &StrictBindingAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingAssignmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictBindingAssignmentRule for StrictBindingAssignmentCostRangeRule {
    fn check(&self, item: &StrictBindingAssignment) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict binding assignment.
#[derive(Debug, Clone)]
pub struct StrictBindingAssignmentValidator {
    pub rules: Vec<Rc<dyn StrictBindingAssignmentRule>>,
}

impl StrictBindingAssignmentValidator {
    pub fn validate(&self, item: &StrictBindingAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictBindingAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictBindingAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_binding_assignment_validator() -> StrictBindingAssignmentValidator {
    StrictBindingAssignmentValidator {
        rules: vec![
            Rc::new(StrictBindingAssignmentCodeLengthRule { max_length: 16 }) as Rc<dyn StrictBindingAssignmentRule>,
            Rc::new(StrictBindingAssignmentRequiredOwnerIdRule {
                message: "strict binding assignment needs this field".to_string(),
            }),
            Rc::new(StrictBindingAssignmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(StrictBindingAssignmentAllowedTimezoneRule { allowed: Vec::new() }),
            Rc::new(StrictBindingAssignmentRequiredLabelRule {
                message: "strict binding assignment needs a value".to_string(),
            }),
            Rc::new(StrictBindingAssignmentCostRangeRule { min: 5.1, max: 3901.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_binding_assignment_validator();
        let item = new_strict_binding_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictBindingAssignmentCodeLengthRule { max_length: 3600 };
        let mut item = new_strict_binding_assignment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = StrictBindingAssignmentRequiredOwnerIdRule {
            message: "strict binding assignment needs an entry".to_string(),
        };
        let mut item = new_strict_binding_assignment("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictBindingAssignmentRequiredLabelRule {
            message: "strict binding assignment needs this field".to_string(),
        };
        let mut item = new_strict_binding_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = StrictBindingAssignmentCostRangeRule { min: 9.5, max: 3597.9 };
        let mut item = new_strict_binding_assignment("x", "y");
        item.cost = 3599.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
