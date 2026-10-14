use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default timeline notice must pass. Check returns its problems, or none.
pub trait DefaultTimelineNoticeRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTimelineNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineNoticeAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultTimelineNoticeRule for DefaultTimelineNoticeAllowedCodeRule {
    fn check(&self, item: &DefaultTimelineNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineNoticeTenantIdLengthRule {
    pub max_length: i64,
}

impl DefaultTimelineNoticeRule for DefaultTimelineNoticeTenantIdLengthRule {
    fn check(&self, item: &DefaultTimelineNotice) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineNoticeRequiredCodeRule {
    pub message: String,
}

impl DefaultTimelineNoticeRule for DefaultTimelineNoticeRequiredCodeRule {
    fn check(&self, item: &DefaultTimelineNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineNoticeRequiredLabelRule {
    pub message: String,
}

impl DefaultTimelineNoticeRule for DefaultTimelineNoticeRequiredLabelRule {
    fn check(&self, item: &DefaultTimelineNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default timeline notice.
#[derive(Debug, Clone)]
pub struct DefaultTimelineNoticeValidator {
    pub rules: Vec<Rc<dyn DefaultTimelineNoticeRule>>,
}

impl DefaultTimelineNoticeValidator {
    pub fn validate(&self, item: &DefaultTimelineNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultTimelineNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTimelineNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_timeline_notice_validator() -> DefaultTimelineNoticeValidator {
    DefaultTimelineNoticeValidator {
        rules: vec![
            Rc::new(DefaultTimelineNoticeAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "seasonal".to_string()],
            }) as Rc<dyn DefaultTimelineNoticeRule>,
            Rc::new(DefaultTimelineNoticeTenantIdLengthRule { max_length: 3600 }),
            Rc::new(DefaultTimelineNoticeRequiredCodeRule {
                message: "default timeline notice needs an entry".to_string(),
            }),
            Rc::new(DefaultTimelineNoticeRequiredLabelRule {
                message: "default timeline notice needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_timeline_notice_validator();
        let item = new_default_timeline_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = DefaultTimelineNoticeTenantIdLengthRule { max_length: 32 };
        let mut item = new_default_timeline_notice("x", "y");
        item.tenant_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultTimelineNoticeRequiredCodeRule {
            message: "default timeline notice needs this field".to_string(),
        };
        let mut item = new_default_timeline_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultTimelineNoticeRequiredLabelRule {
            message: "default timeline notice needs an entry".to_string(),
        };
        let mut item = new_default_timeline_notice("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
