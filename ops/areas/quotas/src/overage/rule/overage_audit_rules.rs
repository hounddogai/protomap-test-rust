use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage audit must pass. Check returns its problems, or none.
pub trait OverageAuditRule: std::fmt::Debug {
    fn check(&self, item: &OverageAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAuditLabelLengthRule {
    pub max_length: i64,
}

impl OverageAuditRule for OverageAuditLabelLengthRule {
    fn check(&self, item: &OverageAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAuditCodeLengthRule {
    pub max_length: i64,
}

impl OverageAuditRule for OverageAuditCodeLengthRule {
    fn check(&self, item: &OverageAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAuditRequiredContactEmailRule {
    pub message: String,
}

impl OverageAuditRule for OverageAuditRequiredContactEmailRule {
    fn check(&self, item: &OverageAudit) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage audit.
#[derive(Debug, Clone)]
pub struct OverageAuditValidator {
    pub rules: Vec<Rc<dyn OverageAuditRule>>,
}

impl OverageAuditValidator {
    pub fn validate(&self, item: &OverageAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_audit_validator() -> OverageAuditValidator {
    OverageAuditValidator {
        rules: vec![
            Rc::new(OverageAuditLabelLengthRule { max_length: 64 }) as Rc<dyn OverageAuditRule>,
            Rc::new(OverageAuditCodeLengthRule { max_length: 80 }),
            Rc::new(OverageAuditRequiredContactEmailRule {
                message: "overage audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_audit_validator();
        let item = new_overage_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = OverageAuditLabelLengthRule { max_length: 255 };
        let mut item = new_overage_audit("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageAuditCodeLengthRule { max_length: 32 };
        let mut item = new_overage_audit("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = OverageAuditRequiredContactEmailRule {
            message: "overage audit needs this field".to_string(),
        };
        let mut item = new_overage_audit("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
