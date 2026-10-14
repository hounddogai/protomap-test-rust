use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal limit digest must pass. Check returns its problems, or none.
pub trait SeasonalLimitDigestRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalLimitDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitDigestRequiredCodeRule {
    pub message: String,
}

impl SeasonalLimitDigestRule for SeasonalLimitDigestRequiredCodeRule {
    fn check(&self, item: &SeasonalLimitDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitDigestChecksumLengthRule {
    pub max_length: i64,
}

impl SeasonalLimitDigestRule for SeasonalLimitDigestChecksumLengthRule {
    fn check(&self, item: &SeasonalLimitDigest) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitDigestAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl SeasonalLimitDigestRule for SeasonalLimitDigestAllowedChecksumRule {
    fn check(&self, item: &SeasonalLimitDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is duplicated", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitDigestCodePrefixRule {
    pub prefix: String,
}

impl SeasonalLimitDigestRule for SeasonalLimitDigestCodePrefixRule {
    fn check(&self, item: &SeasonalLimitDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitDigestRequiredTitleRule {
    pub message: String,
}

impl SeasonalLimitDigestRule for SeasonalLimitDigestRequiredTitleRule {
    fn check(&self, item: &SeasonalLimitDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal limit digest.
#[derive(Debug, Clone)]
pub struct SeasonalLimitDigestValidator {
    pub rules: Vec<Rc<dyn SeasonalLimitDigestRule>>,
}

impl SeasonalLimitDigestValidator {
    pub fn validate(&self, item: &SeasonalLimitDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalLimitDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalLimitDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_limit_digest_validator() -> SeasonalLimitDigestValidator {
    SeasonalLimitDigestValidator {
        rules: vec![
            Rc::new(SeasonalLimitDigestRequiredCodeRule {
                message: "seasonal limit digest needs a value".to_string(),
            }) as Rc<dyn SeasonalLimitDigestRule>,
            Rc::new(SeasonalLimitDigestChecksumLengthRule { max_length: 64 }),
            Rc::new(SeasonalLimitDigestAllowedChecksumRule {
                allowed: vec!["scheduled".to_string(), "strict".to_string(), "tiered".to_string()],
            }),
            Rc::new(SeasonalLimitDigestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SeasonalLimitDigestRequiredTitleRule {
                message: "seasonal limit digest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_limit_digest_validator();
        let item = new_seasonal_limit_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalLimitDigestRequiredCodeRule {
            message: "seasonal limit digest needs an entry".to_string(),
        };
        let mut item = new_seasonal_limit_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = SeasonalLimitDigestChecksumLengthRule { max_length: 64 };
        let mut item = new_seasonal_limit_digest("x", "y");
        item.checksum = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalLimitDigestRequiredTitleRule {
            message: "seasonal limit digest needs an entry".to_string(),
        };
        let mut item = new_seasonal_limit_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
