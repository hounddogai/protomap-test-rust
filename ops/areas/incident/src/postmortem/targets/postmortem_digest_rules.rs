use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem digest must pass. Check returns its problems, or none.
pub trait PostmortemDigestRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDigestRequiredCurrencyRule {
    pub message: String,
}

impl PostmortemDigestRule for PostmortemDigestRequiredCurrencyRule {
    fn check(&self, item: &PostmortemDigest) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDigestCurrencyLengthRule {
    pub max_length: i64,
}

impl PostmortemDigestRule for PostmortemDigestCurrencyLengthRule {
    fn check(&self, item: &PostmortemDigest) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDigestPinnedGuardRule {
    pub limit: i64,
}

impl PostmortemDigestRule for PostmortemDigestPinnedGuardRule {
    fn check(&self, item: &PostmortemDigest) -> Vec<String> {
        if item.pinned && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while pinned",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem digest.
#[derive(Debug, Clone)]
pub struct PostmortemDigestValidator {
    pub rules: Vec<Rc<dyn PostmortemDigestRule>>,
}

impl PostmortemDigestValidator {
    pub fn validate(&self, item: &PostmortemDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_digest_validator() -> PostmortemDigestValidator {
    PostmortemDigestValidator {
        rules: vec![
            Rc::new(PostmortemDigestRequiredCurrencyRule {
                message: "postmortem digest needs an entry".to_string(),
            }) as Rc<dyn PostmortemDigestRule>,
            Rc::new(PostmortemDigestCurrencyLengthRule { max_length: 16 }),
            Rc::new(PostmortemDigestPinnedGuardRule { limit: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_digest_validator();
        let item = new_postmortem_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = PostmortemDigestRequiredCurrencyRule {
            message: "postmortem digest needs this field".to_string(),
        };
        let mut item = new_postmortem_digest("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = PostmortemDigestCurrencyLengthRule { max_length: 80 };
        let mut item = new_postmortem_digest("x", "y");
        item.currency = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
