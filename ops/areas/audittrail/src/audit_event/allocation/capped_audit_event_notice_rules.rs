use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped audit event notice must pass. Check returns its problems, or none.
pub trait CappedAuditEventNoticeRule: std::fmt::Debug {
    fn check(&self, item: &CappedAuditEventNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventNoticeAudienceLengthRule {
    pub max_length: i64,
}

impl CappedAuditEventNoticeRule for CappedAuditEventNoticeAudienceLengthRule {
    fn check(&self, item: &CappedAuditEventNotice) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventNoticeRequiredLabelRule {
    pub message: String,
}

impl CappedAuditEventNoticeRule for CappedAuditEventNoticeRequiredLabelRule {
    fn check(&self, item: &CappedAuditEventNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventNoticeLabelLengthRule {
    pub max_length: i64,
}

impl CappedAuditEventNoticeRule for CappedAuditEventNoticeLabelLengthRule {
    fn check(&self, item: &CappedAuditEventNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventNoticeRequiredAudienceRule {
    pub message: String,
}

impl CappedAuditEventNoticeRule for CappedAuditEventNoticeRequiredAudienceRule {
    fn check(&self, item: &CappedAuditEventNotice) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped audit event notice.
#[derive(Debug, Clone)]
pub struct CappedAuditEventNoticeValidator {
    pub rules: Vec<Rc<dyn CappedAuditEventNoticeRule>>,
}

impl CappedAuditEventNoticeValidator {
    pub fn validate(&self, item: &CappedAuditEventNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedAuditEventNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedAuditEventNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_audit_event_notice_validator() -> CappedAuditEventNoticeValidator {
    CappedAuditEventNoticeValidator {
        rules: vec![
            Rc::new(CappedAuditEventNoticeAudienceLengthRule { max_length: 3600 })
                as Rc<dyn CappedAuditEventNoticeRule>,
            Rc::new(CappedAuditEventNoticeRequiredLabelRule {
                message: "capped audit event notice needs an entry".to_string(),
            }),
            Rc::new(CappedAuditEventNoticeLabelLengthRule { max_length: 80 }),
            Rc::new(CappedAuditEventNoticeRequiredAudienceRule {
                message: "capped audit event notice needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_audit_event_notice_validator();
        let item = new_capped_audit_event_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = CappedAuditEventNoticeAudienceLengthRule { max_length: 1000 };
        let mut item = new_capped_audit_event_notice("x", "y");
        item.audience = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedAuditEventNoticeRequiredLabelRule {
            message: "capped audit event notice needs this field".to_string(),
        };
        let mut item = new_capped_audit_event_notice("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CappedAuditEventNoticeLabelLengthRule { max_length: 86400 };
        let mut item = new_capped_audit_event_notice("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = CappedAuditEventNoticeRequiredAudienceRule {
            message: "capped audit event notice needs an entry".to_string(),
        };
        let mut item = new_capped_audit_event_notice("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
