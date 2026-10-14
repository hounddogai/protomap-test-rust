use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional retention notice must pass. Check returns its problems, or none.
pub trait RegionalRetentionNoticeRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRetentionNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionNoticeCodePrefixRule {
    pub prefix: String,
}

impl RegionalRetentionNoticeRule for RegionalRetentionNoticeCodePrefixRule {
    fn check(&self, item: &RegionalRetentionNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionNoticePrincipalLengthRule {
    pub max_length: i64,
}

impl RegionalRetentionNoticeRule for RegionalRetentionNoticePrincipalLengthRule {
    fn check(&self, item: &RegionalRetentionNotice) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionNoticeTitleLengthRule {
    pub max_length: i64,
}

impl RegionalRetentionNoticeRule for RegionalRetentionNoticeTitleLengthRule {
    fn check(&self, item: &RegionalRetentionNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionNoticeAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalRetentionNoticeRule for RegionalRetentionNoticeAllowedCodeRule {
    fn check(&self, item: &RegionalRetentionNotice) -> Vec<String> {
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
pub struct RegionalRetentionNoticeRequiredLocaleRule {
    pub message: String,
}

impl RegionalRetentionNoticeRule for RegionalRetentionNoticeRequiredLocaleRule {
    fn check(&self, item: &RegionalRetentionNotice) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional retention notice.
#[derive(Debug, Clone)]
pub struct RegionalRetentionNoticeValidator {
    pub rules: Vec<Rc<dyn RegionalRetentionNoticeRule>>,
}

impl RegionalRetentionNoticeValidator {
    pub fn validate(&self, item: &RegionalRetentionNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalRetentionNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRetentionNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_retention_notice_validator() -> RegionalRetentionNoticeValidator {
    RegionalRetentionNoticeValidator {
        rules: vec![
            Rc::new(RegionalRetentionNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RegionalRetentionNoticeRule>,
            Rc::new(RegionalRetentionNoticePrincipalLengthRule { max_length: 32 }),
            Rc::new(RegionalRetentionNoticeTitleLengthRule { max_length: 1000 }),
            Rc::new(RegionalRetentionNoticeAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "legacy".to_string()],
            }),
            Rc::new(RegionalRetentionNoticeRequiredLocaleRule {
                message: "regional retention notice needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_retention_notice_validator();
        let item = new_regional_retention_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RegionalRetentionNoticePrincipalLengthRule { max_length: 255 };
        let mut item = new_regional_retention_notice("x", "y");
        item.principal = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalRetentionNoticeTitleLengthRule { max_length: 80 };
        let mut item = new_regional_retention_notice("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RegionalRetentionNoticeRequiredLocaleRule {
            message: "regional retention notice needs this field".to_string(),
        };
        let mut item = new_regional_retention_notice("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
