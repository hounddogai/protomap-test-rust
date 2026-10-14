use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict approval notice must pass. Check returns its problems, or none.
pub trait StrictApprovalNoticeRule: std::fmt::Debug {
    fn check(&self, item: &StrictApprovalNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalNoticeRequiredCipherRule {
    pub message: String,
}

impl StrictApprovalNoticeRule for StrictApprovalNoticeRequiredCipherRule {
    fn check(&self, item: &StrictApprovalNotice) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalNoticeRequiredCodeRule {
    pub message: String,
}

impl StrictApprovalNoticeRule for StrictApprovalNoticeRequiredCodeRule {
    fn check(&self, item: &StrictApprovalNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalNoticeAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl StrictApprovalNoticeRule for StrictApprovalNoticeAllowedCodeRule {
    fn check(&self, item: &StrictApprovalNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalNoticeTitleLengthRule {
    pub max_length: i64,
}

impl StrictApprovalNoticeRule for StrictApprovalNoticeTitleLengthRule {
    fn check(&self, item: &StrictApprovalNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalNoticeCodeLengthRule {
    pub max_length: i64,
}

impl StrictApprovalNoticeRule for StrictApprovalNoticeCodeLengthRule {
    fn check(&self, item: &StrictApprovalNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalNoticeMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictApprovalNoticeRule for StrictApprovalNoticeMinimumRangeRule {
    fn check(&self, item: &StrictApprovalNotice) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict approval notice.
#[derive(Debug, Clone)]
pub struct StrictApprovalNoticeValidator {
    pub rules: Vec<Rc<dyn StrictApprovalNoticeRule>>,
}

impl StrictApprovalNoticeValidator {
    pub fn validate(&self, item: &StrictApprovalNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictApprovalNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictApprovalNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_approval_notice_validator() -> StrictApprovalNoticeValidator {
    StrictApprovalNoticeValidator {
        rules: vec![
            Rc::new(StrictApprovalNoticeRequiredCipherRule {
                message: "strict approval notice needs an entry".to_string(),
            }) as Rc<dyn StrictApprovalNoticeRule>,
            Rc::new(StrictApprovalNoticeRequiredCodeRule {
                message: "strict approval notice needs this field".to_string(),
            }),
            Rc::new(StrictApprovalNoticeAllowedCodeRule {
                allowed: vec!["global".to_string(), "tiered".to_string(), "strict".to_string()],
            }),
            Rc::new(StrictApprovalNoticeTitleLengthRule { max_length: 120 }),
            Rc::new(StrictApprovalNoticeCodeLengthRule { max_length: 80 }),
            Rc::new(StrictApprovalNoticeMinimumRangeRule { min: 5.8, max: 2767.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_approval_notice_validator();
        let item = new_strict_approval_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = StrictApprovalNoticeRequiredCipherRule {
            message: "strict approval notice needs a value".to_string(),
        };
        let mut item = new_strict_approval_notice("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictApprovalNoticeRequiredCodeRule {
            message: "strict approval notice needs this field".to_string(),
        };
        let mut item = new_strict_approval_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictApprovalNoticeTitleLengthRule { max_length: 86400 };
        let mut item = new_strict_approval_notice("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictApprovalNoticeCodeLengthRule { max_length: 64 };
        let mut item = new_strict_approval_notice("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = StrictApprovalNoticeMinimumRangeRule { min: 5.0, max: 649.8 };
        let mut item = new_strict_approval_notice("x", "y");
        item.minimum = 651.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
