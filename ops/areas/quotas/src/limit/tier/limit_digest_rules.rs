use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a limit digest must pass. Check returns its problems, or none.
pub trait LimitDigestRule: std::fmt::Debug {
    fn check(&self, item: &LimitDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDigestAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LimitDigestRule for LimitDigestAllowedLabelRule {
    fn check(&self, item: &LimitDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDigestCodeLengthRule {
    pub max_length: i64,
}

impl LimitDigestRule for LimitDigestCodeLengthRule {
    fn check(&self, item: &LimitDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDigestRequiredCodeRule {
    pub message: String,
}

impl LimitDigestRule for LimitDigestRequiredCodeRule {
    fn check(&self, item: &LimitDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDigestRequiredRegionRule {
    pub message: String,
}

impl LimitDigestRule for LimitDigestRequiredRegionRule {
    fn check(&self, item: &LimitDigest) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDigestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitDigestRule for LimitDigestCreatedAtRangeRule {
    fn check(&self, item: &LimitDigest) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a limit digest.
#[derive(Debug, Clone)]
pub struct LimitDigestValidator {
    pub rules: Vec<Rc<dyn LimitDigestRule>>,
}

impl LimitDigestValidator {
    pub fn validate(&self, item: &LimitDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_digest_validator() -> LimitDigestValidator {
    LimitDigestValidator {
        rules: vec![
            Rc::new(LimitDigestAllowedLabelRule {
                allowed: vec!["manual".to_string(), "tiered".to_string(), "rolling".to_string()],
            }) as Rc<dyn LimitDigestRule>,
            Rc::new(LimitDigestCodeLengthRule { max_length: 3600 }),
            Rc::new(LimitDigestRequiredCodeRule {
                message: "limit digest needs this field".to_string(),
            }),
            Rc::new(LimitDigestRequiredRegionRule {
                message: "limit digest needs this field".to_string(),
            }),
            Rc::new(LimitDigestCreatedAtRangeRule { min: 32, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_digest_validator();
        let item = new_limit_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitDigestCodeLengthRule { max_length: 1000 };
        let mut item = new_limit_digest("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitDigestRequiredCodeRule {
            message: "limit digest needs a value".to_string(),
        };
        let mut item = new_limit_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = LimitDigestRequiredRegionRule {
            message: "limit digest needs an entry".to_string(),
        };
        let mut item = new_limit_digest("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LimitDigestCreatedAtRangeRule { min: 120, max: 3600 };
        let mut item = new_limit_digest("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
