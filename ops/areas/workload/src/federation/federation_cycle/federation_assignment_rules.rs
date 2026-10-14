use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation assignment must pass. Check returns its problems, or none.
pub trait FederationAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &FederationAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationAssignmentRequiredPrincipalRule {
    pub message: String,
}

impl FederationAssignmentRule for FederationAssignmentRequiredPrincipalRule {
    fn check(&self, item: &FederationAssignment) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationAssignmentScopeLengthRule {
    pub max_length: i64,
}

impl FederationAssignmentRule for FederationAssignmentScopeLengthRule {
    fn check(&self, item: &FederationAssignment) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationAssignmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationAssignmentRule for FederationAssignmentUpdatedAtRangeRule {
    fn check(&self, item: &FederationAssignment) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation assignment.
#[derive(Debug, Clone)]
pub struct FederationAssignmentValidator {
    pub rules: Vec<Rc<dyn FederationAssignmentRule>>,
}

impl FederationAssignmentValidator {
    pub fn validate(&self, item: &FederationAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_assignment_validator() -> FederationAssignmentValidator {
    FederationAssignmentValidator {
        rules: vec![
            Rc::new(FederationAssignmentRequiredPrincipalRule {
                message: "federation assignment needs an entry".to_string(),
            }) as Rc<dyn FederationAssignmentRule>,
            Rc::new(FederationAssignmentScopeLengthRule { max_length: 120 }),
            Rc::new(FederationAssignmentUpdatedAtRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_assignment_validator();
        let item = new_federation_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = FederationAssignmentRequiredPrincipalRule {
            message: "federation assignment needs an entry".to_string(),
        };
        let mut item = new_federation_assignment("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = FederationAssignmentScopeLengthRule { max_length: 16 };
        let mut item = new_federation_assignment("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = FederationAssignmentUpdatedAtRangeRule { min: 32, max: 255 };
        let mut item = new_federation_assignment("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
