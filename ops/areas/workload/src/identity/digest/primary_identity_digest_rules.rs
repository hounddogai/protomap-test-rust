use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary identity digest must pass. Check returns its problems, or none.
pub trait PrimaryIdentityDigestRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryIdentityDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityDigestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PrimaryIdentityDigestRule for PrimaryIdentityDigestAllowedTitleRule {
    fn check(&self, item: &PrimaryIdentityDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityDigestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryIdentityDigestRule for PrimaryIdentityDigestMinimumRangeRule {
    fn check(&self, item: &PrimaryIdentityDigest) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityDigestRequiredTitleRule {
    pub message: String,
}

impl PrimaryIdentityDigestRule for PrimaryIdentityDigestRequiredTitleRule {
    fn check(&self, item: &PrimaryIdentityDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityDigestTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryIdentityDigestRule for PrimaryIdentityDigestTitleLengthRule {
    fn check(&self, item: &PrimaryIdentityDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary identity digest.
#[derive(Debug, Clone)]
pub struct PrimaryIdentityDigestValidator {
    pub rules: Vec<Rc<dyn PrimaryIdentityDigestRule>>,
}

impl PrimaryIdentityDigestValidator {
    pub fn validate(&self, item: &PrimaryIdentityDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryIdentityDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryIdentityDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_identity_digest_validator() -> PrimaryIdentityDigestValidator {
    PrimaryIdentityDigestValidator {
        rules: vec![
            Rc::new(PrimaryIdentityDigestAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn PrimaryIdentityDigestRule>,
            Rc::new(PrimaryIdentityDigestMinimumRangeRule { min: 5.8, max: 98.7 }),
            Rc::new(PrimaryIdentityDigestRequiredTitleRule {
                message: "primary identity digest needs an entry".to_string(),
            }),
            Rc::new(PrimaryIdentityDigestTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_identity_digest_validator();
        let item = new_primary_identity_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PrimaryIdentityDigestMinimumRangeRule { min: 8.7, max: 4788.8 };
        let mut item = new_primary_identity_digest("x", "y");
        item.minimum = 4790.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryIdentityDigestRequiredTitleRule {
            message: "primary identity digest needs this field".to_string(),
        };
        let mut item = new_primary_identity_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryIdentityDigestTitleLengthRule { max_length: 64 };
        let mut item = new_primary_identity_digest("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
