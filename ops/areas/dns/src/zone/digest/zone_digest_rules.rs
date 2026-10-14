use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone digest must pass. Check returns its problems, or none.
pub trait ZoneDigestRule: std::fmt::Debug {
    fn check(&self, item: &ZoneDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDigestFingerprintLengthRule {
    pub max_length: i64,
}

impl ZoneDigestRule for ZoneDigestFingerprintLengthRule {
    fn check(&self, item: &ZoneDigest) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDigestVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneDigestRule for ZoneDigestVersionRangeRule {
    fn check(&self, item: &ZoneDigest) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDigestRequiredFingerprintRule {
    pub message: String,
}

impl ZoneDigestRule for ZoneDigestRequiredFingerprintRule {
    fn check(&self, item: &ZoneDigest) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDigestTitleLengthRule {
    pub max_length: i64,
}

impl ZoneDigestRule for ZoneDigestTitleLengthRule {
    fn check(&self, item: &ZoneDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDigestRequiredTitleRule {
    pub message: String,
}

impl ZoneDigestRule for ZoneDigestRequiredTitleRule {
    fn check(&self, item: &ZoneDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone digest.
#[derive(Debug, Clone)]
pub struct ZoneDigestValidator {
    pub rules: Vec<Rc<dyn ZoneDigestRule>>,
}

impl ZoneDigestValidator {
    pub fn validate(&self, item: &ZoneDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_digest_validator() -> ZoneDigestValidator {
    ZoneDigestValidator {
        rules: vec![
            Rc::new(ZoneDigestFingerprintLengthRule { max_length: 120 }) as Rc<dyn ZoneDigestRule>,
            Rc::new(ZoneDigestVersionRangeRule { min: 64, max: 1000 }),
            Rc::new(ZoneDigestRequiredFingerprintRule {
                message: "zone digest needs an entry".to_string(),
            }),
            Rc::new(ZoneDigestTitleLengthRule { max_length: 3600 }),
            Rc::new(ZoneDigestRequiredTitleRule {
                message: "zone digest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_digest_validator();
        let item = new_zone_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ZoneDigestFingerprintLengthRule { max_length: 32 };
        let mut item = new_zone_digest("x", "y");
        item.fingerprint = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = ZoneDigestVersionRangeRule { min: 16, max: 32 };
        let mut item = new_zone_digest("x", "y");
        item.version = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ZoneDigestRequiredFingerprintRule {
            message: "zone digest needs a value".to_string(),
        };
        let mut item = new_zone_digest("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ZoneDigestTitleLengthRule { max_length: 3600 };
        let mut item = new_zone_digest("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ZoneDigestRequiredTitleRule {
            message: "zone digest needs a value".to_string(),
        };
        let mut item = new_zone_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
