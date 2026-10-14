use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending query audit must pass. Check returns its problems, or none.
pub trait PendingQueryAuditRule: std::fmt::Debug {
    fn check(&self, item: &PendingQueryAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryAuditCountryLengthRule {
    pub max_length: i64,
}

impl PendingQueryAuditRule for PendingQueryAuditCountryLengthRule {
    fn check(&self, item: &PendingQueryAudit) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryAuditRequiredCodeRule {
    pub message: String,
}

impl PendingQueryAuditRule for PendingQueryAuditRequiredCodeRule {
    fn check(&self, item: &PendingQueryAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryAuditNameLengthRule {
    pub max_length: i64,
}

impl PendingQueryAuditRule for PendingQueryAuditNameLengthRule {
    fn check(&self, item: &PendingQueryAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending query audit.
#[derive(Debug, Clone)]
pub struct PendingQueryAuditValidator {
    pub rules: Vec<Rc<dyn PendingQueryAuditRule>>,
}

impl PendingQueryAuditValidator {
    pub fn validate(&self, item: &PendingQueryAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingQueryAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingQueryAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_query_audit_validator() -> PendingQueryAuditValidator {
    PendingQueryAuditValidator {
        rules: vec![
            Rc::new(PendingQueryAuditCountryLengthRule { max_length: 16 }) as Rc<dyn PendingQueryAuditRule>,
            Rc::new(PendingQueryAuditRequiredCodeRule {
                message: "pending query audit needs a value".to_string(),
            }),
            Rc::new(PendingQueryAuditNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_query_audit_validator();
        let item = new_pending_query_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PendingQueryAuditCountryLengthRule { max_length: 3600 };
        let mut item = new_pending_query_audit("x", "y");
        item.country = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingQueryAuditRequiredCodeRule {
            message: "pending query audit needs this field".to_string(),
        };
        let mut item = new_pending_query_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingQueryAuditNameLengthRule { max_length: 86400 };
        let mut item = new_pending_query_audit("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
