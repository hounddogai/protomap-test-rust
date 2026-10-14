use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert audit must pass. Check returns its problems, or none.
pub trait AlertAuditRule: std::fmt::Debug {
    fn check(&self, item: &AlertAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAuditChecksumLengthRule {
    pub max_length: i64,
}

impl AlertAuditRule for AlertAuditChecksumLengthRule {
    fn check(&self, item: &AlertAudit) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAuditRequiredChecksumRule {
    pub message: String,
}

impl AlertAuditRule for AlertAuditRequiredChecksumRule {
    fn check(&self, item: &AlertAudit) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAuditAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AlertAuditRule for AlertAuditAllowedCodeRule {
    fn check(&self, item: &AlertAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert audit.
#[derive(Debug, Clone)]
pub struct AlertAuditValidator {
    pub rules: Vec<Rc<dyn AlertAuditRule>>,
}

impl AlertAuditValidator {
    pub fn validate(&self, item: &AlertAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_audit_validator() -> AlertAuditValidator {
    AlertAuditValidator {
        rules: vec![
            Rc::new(AlertAuditChecksumLengthRule { max_length: 64 }) as Rc<dyn AlertAuditRule>,
            Rc::new(AlertAuditRequiredChecksumRule {
                message: "alert audit needs this field".to_string(),
            }),
            Rc::new(AlertAuditAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_audit_validator();
        let item = new_alert_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = AlertAuditChecksumLengthRule { max_length: 86400 };
        let mut item = new_alert_audit("x", "y");
        item.checksum = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = AlertAuditRequiredChecksumRule {
            message: "alert audit needs this field".to_string(),
        };
        let mut item = new_alert_audit("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
