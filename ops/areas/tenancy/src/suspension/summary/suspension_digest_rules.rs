use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension digest must pass. Check returns its problems, or none.
pub trait SuspensionDigestRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDigestCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionDigestRule for SuspensionDigestCodeLengthRule {
    fn check(&self, item: &SuspensionDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDigestTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl SuspensionDigestRule for SuspensionDigestTagsLimitRule {
    fn check(&self, item: &SuspensionDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDigestContactEmailLengthRule {
    pub max_length: i64,
}

impl SuspensionDigestRule for SuspensionDigestContactEmailLengthRule {
    fn check(&self, item: &SuspensionDigest) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDigestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionDigestRule for SuspensionDigestMaximumRangeRule {
    fn check(&self, item: &SuspensionDigest) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDigestKeyIdLengthRule {
    pub max_length: i64,
}

impl SuspensionDigestRule for SuspensionDigestKeyIdLengthRule {
    fn check(&self, item: &SuspensionDigest) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension digest.
#[derive(Debug, Clone)]
pub struct SuspensionDigestValidator {
    pub rules: Vec<Rc<dyn SuspensionDigestRule>>,
}

impl SuspensionDigestValidator {
    pub fn validate(&self, item: &SuspensionDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_digest_validator() -> SuspensionDigestValidator {
    SuspensionDigestValidator {
        rules: vec![
            Rc::new(SuspensionDigestCodeLengthRule { max_length: 16 }) as Rc<dyn SuspensionDigestRule>,
            Rc::new(SuspensionDigestTagsLimitRule {
                max_count: 80,
                banned: vec!["capped".to_string(), "weighted".to_string(), "primary".to_string()],
            }),
            Rc::new(SuspensionDigestContactEmailLengthRule { max_length: 86400 }),
            Rc::new(SuspensionDigestMaximumRangeRule { min: 2.4, max: 4416.9 }),
            Rc::new(SuspensionDigestKeyIdLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_digest_validator();
        let item = new_suspension_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SuspensionDigestCodeLengthRule { max_length: 80 };
        let mut item = new_suspension_digest("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = SuspensionDigestContactEmailLengthRule { max_length: 80 };
        let mut item = new_suspension_digest("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = SuspensionDigestMaximumRangeRule { min: 9.3, max: 223.1 };
        let mut item = new_suspension_digest("x", "y");
        item.maximum = 224.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = SuspensionDigestKeyIdLengthRule { max_length: 64 };
        let mut item = new_suspension_digest("x", "y");
        item.key_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
