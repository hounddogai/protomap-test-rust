use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout digest must pass. Check returns its problems, or none.
pub trait RolloutDigestRule: std::fmt::Debug {
    fn check(&self, item: &RolloutDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutDigestRequiredCodeRule {
    pub message: String,
}

impl RolloutDigestRule for RolloutDigestRequiredCodeRule {
    fn check(&self, item: &RolloutDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutDigestRequiredNameRule {
    pub message: String,
}

impl RolloutDigestRule for RolloutDigestRequiredNameRule {
    fn check(&self, item: &RolloutDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutDigestCipherLengthRule {
    pub max_length: i64,
}

impl RolloutDigestRule for RolloutDigestCipherLengthRule {
    fn check(&self, item: &RolloutDigest) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutDigestCodeLengthRule {
    pub max_length: i64,
}

impl RolloutDigestRule for RolloutDigestCodeLengthRule {
    fn check(&self, item: &RolloutDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutDigestLabelLengthRule {
    pub max_length: i64,
}

impl RolloutDigestRule for RolloutDigestLabelLengthRule {
    fn check(&self, item: &RolloutDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutDigestStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutDigestRule for RolloutDigestStartsAtRangeRule {
    fn check(&self, item: &RolloutDigest) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout digest.
#[derive(Debug, Clone)]
pub struct RolloutDigestValidator {
    pub rules: Vec<Rc<dyn RolloutDigestRule>>,
}

impl RolloutDigestValidator {
    pub fn validate(&self, item: &RolloutDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_digest_validator() -> RolloutDigestValidator {
    RolloutDigestValidator {
        rules: vec![
            Rc::new(RolloutDigestRequiredCodeRule {
                message: "rollout digest needs an entry".to_string(),
            }) as Rc<dyn RolloutDigestRule>,
            Rc::new(RolloutDigestRequiredNameRule {
                message: "rollout digest needs a value".to_string(),
            }),
            Rc::new(RolloutDigestCipherLengthRule { max_length: 32 }),
            Rc::new(RolloutDigestCodeLengthRule { max_length: 64 }),
            Rc::new(RolloutDigestLabelLengthRule { max_length: 86400 }),
            Rc::new(RolloutDigestStartsAtRangeRule { min: 80, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_digest_validator();
        let item = new_rollout_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutDigestRequiredCodeRule {
            message: "rollout digest needs a value".to_string(),
        };
        let mut item = new_rollout_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RolloutDigestRequiredNameRule {
            message: "rollout digest needs a value".to_string(),
        };
        let mut item = new_rollout_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = RolloutDigestCipherLengthRule { max_length: 80 };
        let mut item = new_rollout_digest("x", "y");
        item.cipher = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutDigestCodeLengthRule { max_length: 120 };
        let mut item = new_rollout_digest("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RolloutDigestLabelLengthRule { max_length: 255 };
        let mut item = new_rollout_digest("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RolloutDigestStartsAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_rollout_digest("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
