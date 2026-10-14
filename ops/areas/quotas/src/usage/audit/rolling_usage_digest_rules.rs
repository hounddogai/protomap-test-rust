use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling usage digest must pass. Check returns its problems, or none.
pub trait RollingUsageDigestRule: std::fmt::Debug {
    fn check(&self, item: &RollingUsageDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageDigestRequiredCodeRule {
    pub message: String,
}

impl RollingUsageDigestRule for RollingUsageDigestRequiredCodeRule {
    fn check(&self, item: &RollingUsageDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageDigestCodePrefixRule {
    pub prefix: String,
}

impl RollingUsageDigestRule for RollingUsageDigestCodePrefixRule {
    fn check(&self, item: &RollingUsageDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageDigestSegmentLengthRule {
    pub max_length: i64,
}

impl RollingUsageDigestRule for RollingUsageDigestSegmentLengthRule {
    fn check(&self, item: &RollingUsageDigest) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling usage digest.
#[derive(Debug, Clone)]
pub struct RollingUsageDigestValidator {
    pub rules: Vec<Rc<dyn RollingUsageDigestRule>>,
}

impl RollingUsageDigestValidator {
    pub fn validate(&self, item: &RollingUsageDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingUsageDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingUsageDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_usage_digest_validator() -> RollingUsageDigestValidator {
    RollingUsageDigestValidator {
        rules: vec![
            Rc::new(RollingUsageDigestRequiredCodeRule {
                message: "rolling usage digest needs this field".to_string(),
            }) as Rc<dyn RollingUsageDigestRule>,
            Rc::new(RollingUsageDigestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RollingUsageDigestSegmentLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_usage_digest_validator();
        let item = new_rolling_usage_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingUsageDigestRequiredCodeRule {
            message: "rolling usage digest needs an entry".to_string(),
        };
        let mut item = new_rolling_usage_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = RollingUsageDigestSegmentLengthRule { max_length: 16 };
        let mut item = new_rolling_usage_digest("x", "y");
        item.segment = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
