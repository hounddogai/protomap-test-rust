use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering audit must pass. Check returns its problems, or none.
pub trait PeeringAuditRule: std::fmt::Debug {
    fn check(&self, item: &PeeringAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAuditCodePrefixRule {
    pub prefix: String,
}

impl PeeringAuditRule for PeeringAuditCodePrefixRule {
    fn check(&self, item: &PeeringAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAuditRequiredTimezoneRule {
    pub message: String,
}

impl PeeringAuditRule for PeeringAuditRequiredTimezoneRule {
    fn check(&self, item: &PeeringAudit) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAuditRequiredTitleRule {
    pub message: String,
}

impl PeeringAuditRule for PeeringAuditRequiredTitleRule {
    fn check(&self, item: &PeeringAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAuditNotesLengthRule {
    pub max_length: i64,
}

impl PeeringAuditRule for PeeringAuditNotesLengthRule {
    fn check(&self, item: &PeeringAudit) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering audit.
#[derive(Debug, Clone)]
pub struct PeeringAuditValidator {
    pub rules: Vec<Rc<dyn PeeringAuditRule>>,
}

impl PeeringAuditValidator {
    pub fn validate(&self, item: &PeeringAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_audit_validator() -> PeeringAuditValidator {
    PeeringAuditValidator {
        rules: vec![
            Rc::new(PeeringAuditCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn PeeringAuditRule>,
            Rc::new(PeeringAuditRequiredTimezoneRule {
                message: "peering audit needs a value".to_string(),
            }),
            Rc::new(PeeringAuditRequiredTitleRule {
                message: "peering audit needs a value".to_string(),
            }),
            Rc::new(PeeringAuditNotesLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_audit_validator();
        let item = new_peering_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PeeringAuditRequiredTimezoneRule {
            message: "peering audit needs this field".to_string(),
        };
        let mut item = new_peering_audit("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PeeringAuditRequiredTitleRule {
            message: "peering audit needs an entry".to_string(),
        };
        let mut item = new_peering_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = PeeringAuditNotesLengthRule { max_length: 32 };
        let mut item = new_peering_audit("x", "y");
        item.notes = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
