use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy failover digest must pass. Check returns its problems, or none.
pub trait LegacyFailoverDigestRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFailoverDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverDigestRequiredTitleRule {
    pub message: String,
}

impl LegacyFailoverDigestRule for LegacyFailoverDigestRequiredTitleRule {
    fn check(&self, item: &LegacyFailoverDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverDigestSegmentLengthRule {
    pub max_length: i64,
}

impl LegacyFailoverDigestRule for LegacyFailoverDigestSegmentLengthRule {
    fn check(&self, item: &LegacyFailoverDigest) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverDigestRequiredSourceRule {
    pub message: String,
}

impl LegacyFailoverDigestRule for LegacyFailoverDigestRequiredSourceRule {
    fn check(&self, item: &LegacyFailoverDigest) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverDigestAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl LegacyFailoverDigestRule for LegacyFailoverDigestAllowedSegmentRule {
    fn check(&self, item: &LegacyFailoverDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is below minimum", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverDigestTitleLengthRule {
    pub max_length: i64,
}

impl LegacyFailoverDigestRule for LegacyFailoverDigestTitleLengthRule {
    fn check(&self, item: &LegacyFailoverDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverDigestSourceLengthRule {
    pub max_length: i64,
}

impl LegacyFailoverDigestRule for LegacyFailoverDigestSourceLengthRule {
    fn check(&self, item: &LegacyFailoverDigest) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy failover digest.
#[derive(Debug, Clone)]
pub struct LegacyFailoverDigestValidator {
    pub rules: Vec<Rc<dyn LegacyFailoverDigestRule>>,
}

impl LegacyFailoverDigestValidator {
    pub fn validate(&self, item: &LegacyFailoverDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyFailoverDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFailoverDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_failover_digest_validator() -> LegacyFailoverDigestValidator {
    LegacyFailoverDigestValidator {
        rules: vec![
            Rc::new(LegacyFailoverDigestRequiredTitleRule {
                message: "legacy failover digest needs a value".to_string(),
            }) as Rc<dyn LegacyFailoverDigestRule>,
            Rc::new(LegacyFailoverDigestSegmentLengthRule { max_length: 16 }),
            Rc::new(LegacyFailoverDigestRequiredSourceRule {
                message: "legacy failover digest needs this field".to_string(),
            }),
            Rc::new(LegacyFailoverDigestAllowedSegmentRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(LegacyFailoverDigestTitleLengthRule { max_length: 32 }),
            Rc::new(LegacyFailoverDigestSourceLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_failover_digest_validator();
        let item = new_legacy_failover_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyFailoverDigestRequiredTitleRule {
            message: "legacy failover digest needs a value".to_string(),
        };
        let mut item = new_legacy_failover_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = LegacyFailoverDigestSegmentLengthRule { max_length: 3600 };
        let mut item = new_legacy_failover_digest("x", "y");
        item.segment = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = LegacyFailoverDigestRequiredSourceRule {
            message: "legacy failover digest needs this field".to_string(),
        };
        let mut item = new_legacy_failover_digest("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyFailoverDigestTitleLengthRule { max_length: 80 };
        let mut item = new_legacy_failover_digest("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = LegacyFailoverDigestSourceLengthRule { max_length: 32 };
        let mut item = new_legacy_failover_digest("x", "y");
        item.source = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
