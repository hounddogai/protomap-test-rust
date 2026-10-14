use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event notice must pass. Check returns its problems, or none.
pub trait AuditEventNoticeRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventNoticeRequiredCodeRule {
    pub message: String,
}

impl AuditEventNoticeRule for AuditEventNoticeRequiredCodeRule {
    fn check(&self, item: &AuditEventNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventNoticeCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventNoticeRule for AuditEventNoticeCodeLengthRule {
    fn check(&self, item: &AuditEventNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventNoticeCodePrefixRule {
    pub prefix: String,
}

impl AuditEventNoticeRule for AuditEventNoticeCodePrefixRule {
    fn check(&self, item: &AuditEventNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventNoticeRequiredSummaryRule {
    pub message: String,
}

impl AuditEventNoticeRule for AuditEventNoticeRequiredSummaryRule {
    fn check(&self, item: &AuditEventNotice) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventNoticeSummaryLengthRule {
    pub max_length: i64,
}

impl AuditEventNoticeRule for AuditEventNoticeSummaryLengthRule {
    fn check(&self, item: &AuditEventNotice) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event notice.
#[derive(Debug, Clone)]
pub struct AuditEventNoticeValidator {
    pub rules: Vec<Rc<dyn AuditEventNoticeRule>>,
}

impl AuditEventNoticeValidator {
    pub fn validate(&self, item: &AuditEventNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_notice_validator() -> AuditEventNoticeValidator {
    AuditEventNoticeValidator {
        rules: vec![
            Rc::new(AuditEventNoticeRequiredCodeRule {
                message: "audit event notice needs an entry".to_string(),
            }) as Rc<dyn AuditEventNoticeRule>,
            Rc::new(AuditEventNoticeCodeLengthRule { max_length: 80 }),
            Rc::new(AuditEventNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AuditEventNoticeRequiredSummaryRule {
                message: "audit event notice needs this field".to_string(),
            }),
            Rc::new(AuditEventNoticeSummaryLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_notice_validator();
        let item = new_audit_event_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventNoticeRequiredCodeRule {
            message: "audit event notice needs a value".to_string(),
        };
        let mut item = new_audit_event_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AuditEventNoticeCodeLengthRule { max_length: 32 };
        let mut item = new_audit_event_notice("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = AuditEventNoticeRequiredSummaryRule {
            message: "audit event notice needs this field".to_string(),
        };
        let mut item = new_audit_event_notice("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = AuditEventNoticeSummaryLengthRule { max_length: 255 };
        let mut item = new_audit_event_notice("x", "y");
        item.summary = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
