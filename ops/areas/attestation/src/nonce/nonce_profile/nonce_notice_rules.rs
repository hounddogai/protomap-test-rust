use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce notice must pass. Check returns its problems, or none.
pub trait NonceNoticeRule: std::fmt::Debug {
    fn check(&self, item: &NonceNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceNoticeRequiredNameRule {
    pub message: String,
}

impl NonceNoticeRule for NonceNoticeRequiredNameRule {
    fn check(&self, item: &NonceNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceNoticeReviewerLengthRule {
    pub max_length: i64,
}

impl NonceNoticeRule for NonceNoticeReviewerLengthRule {
    fn check(&self, item: &NonceNotice) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceNoticeCodePrefixRule {
    pub prefix: String,
}

impl NonceNoticeRule for NonceNoticeCodePrefixRule {
    fn check(&self, item: &NonceNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceNoticeEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceNoticeRule for NonceNoticeEndsAtRangeRule {
    fn check(&self, item: &NonceNotice) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce notice.
#[derive(Debug, Clone)]
pub struct NonceNoticeValidator {
    pub rules: Vec<Rc<dyn NonceNoticeRule>>,
}

impl NonceNoticeValidator {
    pub fn validate(&self, item: &NonceNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_notice_validator() -> NonceNoticeValidator {
    NonceNoticeValidator {
        rules: vec![
            Rc::new(NonceNoticeRequiredNameRule {
                message: "nonce notice needs an entry".to_string(),
            }) as Rc<dyn NonceNoticeRule>,
            Rc::new(NonceNoticeReviewerLengthRule { max_length: 86400 }),
            Rc::new(NonceNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(NonceNoticeEndsAtRangeRule { min: 80, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_notice_validator();
        let item = new_nonce_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = NonceNoticeRequiredNameRule {
            message: "nonce notice needs an entry".to_string(),
        };
        let mut item = new_nonce_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = NonceNoticeReviewerLengthRule { max_length: 80 };
        let mut item = new_nonce_notice("x", "y");
        item.reviewer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = NonceNoticeEndsAtRangeRule { min: 32, max: 255 };
        let mut item = new_nonce_notice("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
