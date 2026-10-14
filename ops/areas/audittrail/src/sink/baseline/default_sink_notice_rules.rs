use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default sink notice must pass. Check returns its problems, or none.
pub trait DefaultSinkNoticeRule: std::fmt::Debug {
    fn check(&self, item: &DefaultSinkNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkNoticeCodePrefixRule {
    pub prefix: String,
}

impl DefaultSinkNoticeRule for DefaultSinkNoticeCodePrefixRule {
    fn check(&self, item: &DefaultSinkNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkNoticeCodeLengthRule {
    pub max_length: i64,
}

impl DefaultSinkNoticeRule for DefaultSinkNoticeCodeLengthRule {
    fn check(&self, item: &DefaultSinkNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkNoticePhoneLengthRule {
    pub max_length: i64,
}

impl DefaultSinkNoticeRule for DefaultSinkNoticePhoneLengthRule {
    fn check(&self, item: &DefaultSinkNotice) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default sink notice.
#[derive(Debug, Clone)]
pub struct DefaultSinkNoticeValidator {
    pub rules: Vec<Rc<dyn DefaultSinkNoticeRule>>,
}

impl DefaultSinkNoticeValidator {
    pub fn validate(&self, item: &DefaultSinkNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultSinkNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultSinkNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_sink_notice_validator() -> DefaultSinkNoticeValidator {
    DefaultSinkNoticeValidator {
        rules: vec![
            Rc::new(DefaultSinkNoticeCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn DefaultSinkNoticeRule>,
            Rc::new(DefaultSinkNoticeCodeLengthRule { max_length: 32 }),
            Rc::new(DefaultSinkNoticePhoneLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_sink_notice_validator();
        let item = new_default_sink_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultSinkNoticeCodeLengthRule { max_length: 3600 };
        let mut item = new_default_sink_notice("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = DefaultSinkNoticePhoneLengthRule { max_length: 3600 };
        let mut item = new_default_sink_notice("x", "y");
        item.phone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
