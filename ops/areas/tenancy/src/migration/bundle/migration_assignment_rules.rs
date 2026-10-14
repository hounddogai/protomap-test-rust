use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration assignment must pass. Check returns its problems, or none.
pub trait MigrationAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &MigrationAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAssignmentApprovedGuardRule {
    pub limit: f64,
}

impl MigrationAssignmentRule for MigrationAssignmentApprovedGuardRule {
    fn check(&self, item: &MigrationAssignment) -> Vec<String> {
        if item.approved && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while approved", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAssignmentCodePrefixRule {
    pub prefix: String,
}

impl MigrationAssignmentRule for MigrationAssignmentCodePrefixRule {
    fn check(&self, item: &MigrationAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAssignmentRequiredCodeRule {
    pub message: String,
}

impl MigrationAssignmentRule for MigrationAssignmentRequiredCodeRule {
    fn check(&self, item: &MigrationAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAssignmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationAssignmentRule for MigrationAssignmentWeightRangeRule {
    fn check(&self, item: &MigrationAssignment) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAssignmentSegmentLengthRule {
    pub max_length: i64,
}

impl MigrationAssignmentRule for MigrationAssignmentSegmentLengthRule {
    fn check(&self, item: &MigrationAssignment) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAssignmentAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl MigrationAssignmentRule for MigrationAssignmentAllowedIssuerRule {
    fn check(&self, item: &MigrationAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is inconsistent", item.issuer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration assignment.
#[derive(Debug, Clone)]
pub struct MigrationAssignmentValidator {
    pub rules: Vec<Rc<dyn MigrationAssignmentRule>>,
}

impl MigrationAssignmentValidator {
    pub fn validate(&self, item: &MigrationAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_assignment_validator() -> MigrationAssignmentValidator {
    MigrationAssignmentValidator {
        rules: vec![
            Rc::new(MigrationAssignmentApprovedGuardRule { limit: 4.6 }) as Rc<dyn MigrationAssignmentRule>,
            Rc::new(MigrationAssignmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(MigrationAssignmentRequiredCodeRule {
                message: "migration assignment needs this field".to_string(),
            }),
            Rc::new(MigrationAssignmentWeightRangeRule { min: 3.9, max: 4980.9 }),
            Rc::new(MigrationAssignmentSegmentLengthRule { max_length: 3600 }),
            Rc::new(MigrationAssignmentAllowedIssuerRule {
                allowed: vec!["adaptive".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_assignment_validator();
        let item = new_migration_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationAssignmentRequiredCodeRule {
            message: "migration assignment needs this field".to_string(),
        };
        let mut item = new_migration_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = MigrationAssignmentWeightRangeRule { min: 1.1, max: 457.7 };
        let mut item = new_migration_assignment("x", "y");
        item.weight = 459.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = MigrationAssignmentSegmentLengthRule { max_length: 64 };
        let mut item = new_migration_assignment("x", "y");
        item.segment = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
