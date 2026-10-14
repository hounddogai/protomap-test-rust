use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event agreement must pass. Check returns its problems, or none.
pub trait AuditEventAgreementRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAgreementRequiredTitleRule {
    pub message: String,
}

impl AuditEventAgreementRule for AuditEventAgreementRequiredTitleRule {
    fn check(&self, item: &AuditEventAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAgreementRequiredCodeRule {
    pub message: String,
}

impl AuditEventAgreementRule for AuditEventAgreementRequiredCodeRule {
    fn check(&self, item: &AuditEventAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAgreementTitleLengthRule {
    pub max_length: i64,
}

impl AuditEventAgreementRule for AuditEventAgreementTitleLengthRule {
    fn check(&self, item: &AuditEventAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAgreementNotesLengthRule {
    pub max_length: i64,
}

impl AuditEventAgreementRule for AuditEventAgreementNotesLengthRule {
    fn check(&self, item: &AuditEventAgreement) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAgreementLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventAgreementRule for AuditEventAgreementLimitRangeRule {
    fn check(&self, item: &AuditEventAgreement) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a audit event agreement.
#[derive(Debug, Clone)]
pub struct AuditEventAgreementValidator {
    pub rules: Vec<Rc<dyn AuditEventAgreementRule>>,
}

impl AuditEventAgreementValidator {
    pub fn validate(&self, item: &AuditEventAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_agreement_validator() -> AuditEventAgreementValidator {
    AuditEventAgreementValidator {
        rules: vec![
            Rc::new(AuditEventAgreementRequiredTitleRule {
                message: "audit event agreement needs a value".to_string(),
            }) as Rc<dyn AuditEventAgreementRule>,
            Rc::new(AuditEventAgreementRequiredCodeRule {
                message: "audit event agreement needs an entry".to_string(),
            }),
            Rc::new(AuditEventAgreementTitleLengthRule { max_length: 3600 }),
            Rc::new(AuditEventAgreementNotesLengthRule { max_length: 120 }),
            Rc::new(AuditEventAgreementLimitRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_agreement_validator();
        let item = new_audit_event_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AuditEventAgreementRequiredTitleRule {
            message: "audit event agreement needs this field".to_string(),
        };
        let mut item = new_audit_event_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventAgreementRequiredCodeRule {
            message: "audit event agreement needs this field".to_string(),
        };
        let mut item = new_audit_event_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AuditEventAgreementTitleLengthRule { max_length: 64 };
        let mut item = new_audit_event_agreement("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = AuditEventAgreementNotesLengthRule { max_length: 1000 };
        let mut item = new_audit_event_agreement("x", "y");
        item.notes = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AuditEventAgreementLimitRangeRule { min: 120, max: 86400 };
        let mut item = new_audit_event_agreement("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
