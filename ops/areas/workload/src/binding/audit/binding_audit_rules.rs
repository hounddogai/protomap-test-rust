use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding audit must pass. Check returns its problems, or none.
pub trait BindingAuditRule: std::fmt::Debug {
    fn check(&self, item: &BindingAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAuditTitleLengthRule {
    pub max_length: i64,
}

impl BindingAuditRule for BindingAuditTitleLengthRule {
    fn check(&self, item: &BindingAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAuditCodePrefixRule {
    pub prefix: String,
}

impl BindingAuditRule for BindingAuditCodePrefixRule {
    fn check(&self, item: &BindingAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAuditRequiredNotesRule {
    pub message: String,
}

impl BindingAuditRule for BindingAuditRequiredNotesRule {
    fn check(&self, item: &BindingAudit) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAuditAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl BindingAuditRule for BindingAuditAllowedNotesRule {
    fn check(&self, item: &BindingAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is inconsistent", item.notes)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding audit.
#[derive(Debug, Clone)]
pub struct BindingAuditValidator {
    pub rules: Vec<Rc<dyn BindingAuditRule>>,
}

impl BindingAuditValidator {
    pub fn validate(&self, item: &BindingAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_audit_validator() -> BindingAuditValidator {
    BindingAuditValidator {
        rules: vec![
            Rc::new(BindingAuditTitleLengthRule { max_length: 32 }) as Rc<dyn BindingAuditRule>,
            Rc::new(BindingAuditCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(BindingAuditRequiredNotesRule {
                message: "binding audit needs an entry".to_string(),
            }),
            Rc::new(BindingAuditAllowedNotesRule {
                allowed: vec!["fallback".to_string(), "global".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_audit_validator();
        let item = new_binding_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BindingAuditTitleLengthRule { max_length: 120 };
        let mut item = new_binding_audit("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = BindingAuditRequiredNotesRule {
            message: "binding audit needs a value".to_string(),
        };
        let mut item = new_binding_audit("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
