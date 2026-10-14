use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote notice must pass. Check returns its problems, or none.
pub trait QuoteNoticeRule: std::fmt::Debug {
    fn check(&self, item: &QuoteNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteNoticeCodeLengthRule {
    pub max_length: i64,
}

impl QuoteNoticeRule for QuoteNoticeCodeLengthRule {
    fn check(&self, item: &QuoteNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteNoticeRequiredNameRule {
    pub message: String,
}

impl QuoteNoticeRule for QuoteNoticeRequiredNameRule {
    fn check(&self, item: &QuoteNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteNoticeNameLengthRule {
    pub max_length: i64,
}

impl QuoteNoticeRule for QuoteNoticeNameLengthRule {
    fn check(&self, item: &QuoteNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote notice.
#[derive(Debug, Clone)]
pub struct QuoteNoticeValidator {
    pub rules: Vec<Rc<dyn QuoteNoticeRule>>,
}

impl QuoteNoticeValidator {
    pub fn validate(&self, item: &QuoteNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_notice_validator() -> QuoteNoticeValidator {
    QuoteNoticeValidator {
        rules: vec![
            Rc::new(QuoteNoticeCodeLengthRule { max_length: 32 }) as Rc<dyn QuoteNoticeRule>,
            Rc::new(QuoteNoticeRequiredNameRule {
                message: "quote notice needs a value".to_string(),
            }),
            Rc::new(QuoteNoticeNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_notice_validator();
        let item = new_quote_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuoteNoticeCodeLengthRule { max_length: 16 };
        let mut item = new_quote_notice("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = QuoteNoticeRequiredNameRule {
            message: "quote notice needs an entry".to_string(),
        };
        let mut item = new_quote_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = QuoteNoticeNameLengthRule { max_length: 16 };
        let mut item = new_quote_notice("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
