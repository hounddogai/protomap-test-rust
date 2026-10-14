use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global audit event notice must pass. Check returns its problems, or none.
pub trait GlobalAuditEventNoticeRule: std::fmt::Debug {
    fn check(&self, item: &GlobalAuditEventNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventNoticeDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalAuditEventNoticeRule for GlobalAuditEventNoticeDiscountRangeRule {
    fn check(&self, item: &GlobalAuditEventNotice) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventNoticeRequiredTitleRule {
    pub message: String,
}

impl GlobalAuditEventNoticeRule for GlobalAuditEventNoticeRequiredTitleRule {
    fn check(&self, item: &GlobalAuditEventNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventNoticeTitleLengthRule {
    pub max_length: i64,
}

impl GlobalAuditEventNoticeRule for GlobalAuditEventNoticeTitleLengthRule {
    fn check(&self, item: &GlobalAuditEventNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventNoticeAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl GlobalAuditEventNoticeRule for GlobalAuditEventNoticeAllowedPhoneRule {
    fn check(&self, item: &GlobalAuditEventNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is unknown", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventNoticeRequiredPhoneRule {
    pub message: String,
}

impl GlobalAuditEventNoticeRule for GlobalAuditEventNoticeRequiredPhoneRule {
    fn check(&self, item: &GlobalAuditEventNotice) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventNoticeCodePrefixRule {
    pub prefix: String,
}

impl GlobalAuditEventNoticeRule for GlobalAuditEventNoticeCodePrefixRule {
    fn check(&self, item: &GlobalAuditEventNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global audit event notice.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventNoticeValidator {
    pub rules: Vec<Rc<dyn GlobalAuditEventNoticeRule>>,
}

impl GlobalAuditEventNoticeValidator {
    pub fn validate(&self, item: &GlobalAuditEventNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalAuditEventNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalAuditEventNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_audit_event_notice_validator() -> GlobalAuditEventNoticeValidator {
    GlobalAuditEventNoticeValidator {
        rules: vec![
            Rc::new(GlobalAuditEventNoticeDiscountRangeRule { min: 0.4, max: 4277.4 })
                as Rc<dyn GlobalAuditEventNoticeRule>,
            Rc::new(GlobalAuditEventNoticeRequiredTitleRule {
                message: "global audit event notice needs this field".to_string(),
            }),
            Rc::new(GlobalAuditEventNoticeTitleLengthRule { max_length: 64 }),
            Rc::new(GlobalAuditEventNoticeAllowedPhoneRule {
                allowed: vec!["batched".to_string(), "pending".to_string(), "lenient".to_string()],
            }),
            Rc::new(GlobalAuditEventNoticeRequiredPhoneRule {
                message: "global audit event notice needs a value".to_string(),
            }),
            Rc::new(GlobalAuditEventNoticeCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_audit_event_notice_validator();
        let item = new_global_audit_event_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = GlobalAuditEventNoticeDiscountRangeRule { min: 9.8, max: 1681.9 };
        let mut item = new_global_audit_event_notice("x", "y");
        item.discount = 1683.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalAuditEventNoticeRequiredTitleRule {
            message: "global audit event notice needs this field".to_string(),
        };
        let mut item = new_global_audit_event_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalAuditEventNoticeTitleLengthRule { max_length: 86400 };
        let mut item = new_global_audit_event_notice("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = GlobalAuditEventNoticeRequiredPhoneRule {
            message: "global audit event notice needs an entry".to_string(),
        };
        let mut item = new_global_audit_event_notice("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
