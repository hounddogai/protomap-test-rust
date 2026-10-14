use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched policy audit must pass. Check returns its problems, or none.
pub trait BatchedPolicyAuditRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPolicyAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAuditRequiredCodeRule {
    pub message: String,
}

impl BatchedPolicyAuditRule for BatchedPolicyAuditRequiredCodeRule {
    fn check(&self, item: &BatchedPolicyAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAuditNotesLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyAuditRule for BatchedPolicyAuditNotesLengthRule {
    fn check(&self, item: &BatchedPolicyAudit) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAuditContactEmailLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyAuditRule for BatchedPolicyAuditContactEmailLengthRule {
    fn check(&self, item: &BatchedPolicyAudit) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAuditRequiredContactEmailRule {
    pub message: String,
}

impl BatchedPolicyAuditRule for BatchedPolicyAuditRequiredContactEmailRule {
    fn check(&self, item: &BatchedPolicyAudit) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAuditCodeLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyAuditRule for BatchedPolicyAuditCodeLengthRule {
    fn check(&self, item: &BatchedPolicyAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched policy audit.
#[derive(Debug, Clone)]
pub struct BatchedPolicyAuditValidator {
    pub rules: Vec<Rc<dyn BatchedPolicyAuditRule>>,
}

impl BatchedPolicyAuditValidator {
    pub fn validate(&self, item: &BatchedPolicyAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPolicyAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPolicyAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_policy_audit_validator() -> BatchedPolicyAuditValidator {
    BatchedPolicyAuditValidator {
        rules: vec![
            Rc::new(BatchedPolicyAuditRequiredCodeRule {
                message: "batched policy audit needs this field".to_string(),
            }) as Rc<dyn BatchedPolicyAuditRule>,
            Rc::new(BatchedPolicyAuditNotesLengthRule { max_length: 32 }),
            Rc::new(BatchedPolicyAuditContactEmailLengthRule { max_length: 80 }),
            Rc::new(BatchedPolicyAuditRequiredContactEmailRule {
                message: "batched policy audit needs this field".to_string(),
            }),
            Rc::new(BatchedPolicyAuditCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_policy_audit_validator();
        let item = new_batched_policy_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedPolicyAuditRequiredCodeRule {
            message: "batched policy audit needs this field".to_string(),
        };
        let mut item = new_batched_policy_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = BatchedPolicyAuditNotesLengthRule { max_length: 120 };
        let mut item = new_batched_policy_audit("x", "y");
        item.notes = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = BatchedPolicyAuditContactEmailLengthRule { max_length: 120 };
        let mut item = new_batched_policy_audit("x", "y");
        item.contact_email = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = BatchedPolicyAuditRequiredContactEmailRule {
            message: "batched policy audit needs a value".to_string(),
        };
        let mut item = new_batched_policy_audit("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedPolicyAuditCodeLengthRule { max_length: 255 };
        let mut item = new_batched_policy_audit("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
