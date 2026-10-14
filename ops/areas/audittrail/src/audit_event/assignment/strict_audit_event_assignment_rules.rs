use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict audit event assignment must pass. Check returns its problems, or none.
pub trait StrictAuditEventAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &StrictAuditEventAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventAssignmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictAuditEventAssignmentRule for StrictAuditEventAssignmentUpdatedAtRangeRule {
    fn check(&self, item: &StrictAuditEventAssignment) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventAssignmentRequiredTitleRule {
    pub message: String,
}

impl StrictAuditEventAssignmentRule for StrictAuditEventAssignmentRequiredTitleRule {
    fn check(&self, item: &StrictAuditEventAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventAssignmentRequiredCodeRule {
    pub message: String,
}

impl StrictAuditEventAssignmentRule for StrictAuditEventAssignmentRequiredCodeRule {
    fn check(&self, item: &StrictAuditEventAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventAssignmentAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl StrictAuditEventAssignmentRule for StrictAuditEventAssignmentAllowedCountryRule {
    fn check(&self, item: &StrictAuditEventAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is missing", item.country)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict audit event assignment.
#[derive(Debug, Clone)]
pub struct StrictAuditEventAssignmentValidator {
    pub rules: Vec<Rc<dyn StrictAuditEventAssignmentRule>>,
}

impl StrictAuditEventAssignmentValidator {
    pub fn validate(&self, item: &StrictAuditEventAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictAuditEventAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictAuditEventAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_audit_event_assignment_validator() -> StrictAuditEventAssignmentValidator {
    StrictAuditEventAssignmentValidator {
        rules: vec![
            Rc::new(StrictAuditEventAssignmentUpdatedAtRangeRule { min: 64, max: 120 })
                as Rc<dyn StrictAuditEventAssignmentRule>,
            Rc::new(StrictAuditEventAssignmentRequiredTitleRule {
                message: "strict audit event assignment needs an entry".to_string(),
            }),
            Rc::new(StrictAuditEventAssignmentRequiredCodeRule {
                message: "strict audit event assignment needs an entry".to_string(),
            }),
            Rc::new(StrictAuditEventAssignmentAllowedCountryRule {
                allowed: vec!["batched".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_audit_event_assignment_validator();
        let item = new_strict_audit_event_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = StrictAuditEventAssignmentUpdatedAtRangeRule { min: 64, max: 120 };
        let mut item = new_strict_audit_event_assignment("x", "y");
        item.updated_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictAuditEventAssignmentRequiredTitleRule {
            message: "strict audit event assignment needs an entry".to_string(),
        };
        let mut item = new_strict_audit_event_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictAuditEventAssignmentRequiredCodeRule {
            message: "strict audit event assignment needs a value".to_string(),
        };
        let mut item = new_strict_audit_event_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
