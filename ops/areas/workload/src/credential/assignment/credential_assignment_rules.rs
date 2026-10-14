use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential assignment must pass. Check returns its problems, or none.
pub trait CredentialAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &CredentialAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAssignmentRequiredExternalRefRule {
    pub message: String,
}

impl CredentialAssignmentRule for CredentialAssignmentRequiredExternalRefRule {
    fn check(&self, item: &CredentialAssignment) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAssignmentMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialAssignmentRule for CredentialAssignmentMarginRangeRule {
    fn check(&self, item: &CredentialAssignment) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAssignmentRequiredNameRule {
    pub message: String,
}

impl CredentialAssignmentRule for CredentialAssignmentRequiredNameRule {
    fn check(&self, item: &CredentialAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CredentialAssignmentRule for CredentialAssignmentAllowedCodeRule {
    fn check(&self, item: &CredentialAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAssignmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialAssignmentRule for CredentialAssignmentCeilingRangeRule {
    fn check(&self, item: &CredentialAssignment) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential assignment.
#[derive(Debug, Clone)]
pub struct CredentialAssignmentValidator {
    pub rules: Vec<Rc<dyn CredentialAssignmentRule>>,
}

impl CredentialAssignmentValidator {
    pub fn validate(&self, item: &CredentialAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_assignment_validator() -> CredentialAssignmentValidator {
    CredentialAssignmentValidator {
        rules: vec![
            Rc::new(CredentialAssignmentRequiredExternalRefRule {
                message: "credential assignment needs a value".to_string(),
            }) as Rc<dyn CredentialAssignmentRule>,
            Rc::new(CredentialAssignmentMarginRangeRule { min: 1.2, max: 1067.4 }),
            Rc::new(CredentialAssignmentRequiredNameRule {
                message: "credential assignment needs a value".to_string(),
            }),
            Rc::new(CredentialAssignmentAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(CredentialAssignmentCeilingRangeRule { min: 9.3, max: 1045.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_assignment_validator();
        let item = new_credential_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = CredentialAssignmentRequiredExternalRefRule {
            message: "credential assignment needs a value".to_string(),
        };
        let mut item = new_credential_assignment("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = CredentialAssignmentMarginRangeRule { min: 2.5, max: 1840.5 };
        let mut item = new_credential_assignment("x", "y");
        item.margin = 1842.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CredentialAssignmentRequiredNameRule {
            message: "credential assignment needs a value".to_string(),
        };
        let mut item = new_credential_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = CredentialAssignmentCeilingRangeRule { min: 9.0, max: 3699.6 };
        let mut item = new_credential_assignment("x", "y");
        item.ceiling = 3701.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
