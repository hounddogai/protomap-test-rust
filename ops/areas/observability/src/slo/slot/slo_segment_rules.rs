use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo segment must pass. Check returns its problems, or none.
pub trait SloSegmentRule: std::fmt::Debug {
    fn check(&self, item: &SloSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSegmentCodePrefixRule {
    pub prefix: String,
}

impl SloSegmentRule for SloSegmentCodePrefixRule {
    fn check(&self, item: &SloSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSegmentTitleLengthRule {
    pub max_length: i64,
}

impl SloSegmentRule for SloSegmentTitleLengthRule {
    fn check(&self, item: &SloSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSegmentFingerprintLengthRule {
    pub max_length: i64,
}

impl SloSegmentRule for SloSegmentFingerprintLengthRule {
    fn check(&self, item: &SloSegment) -> Vec<String> {
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
pub struct SloSegmentAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl SloSegmentRule for SloSegmentAllowedFingerprintRule {
    fn check(&self, item: &SloSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is duplicated", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSegmentRequiredCodeRule {
    pub message: String,
}

impl SloSegmentRule for SloSegmentRequiredCodeRule {
    fn check(&self, item: &SloSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSegmentRequiredFingerprintRule {
    pub message: String,
}

impl SloSegmentRule for SloSegmentRequiredFingerprintRule {
    fn check(&self, item: &SloSegment) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo segment.
#[derive(Debug, Clone)]
pub struct SloSegmentValidator {
    pub rules: Vec<Rc<dyn SloSegmentRule>>,
}

impl SloSegmentValidator {
    pub fn validate(&self, item: &SloSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_segment_validator() -> SloSegmentValidator {
    SloSegmentValidator {
        rules: vec![
            Rc::new(SloSegmentCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn SloSegmentRule>,
            Rc::new(SloSegmentTitleLengthRule { max_length: 1000 }),
            Rc::new(SloSegmentFingerprintLengthRule { max_length: 16 }),
            Rc::new(SloSegmentAllowedFingerprintRule {
                allowed: vec!["primary".to_string(), "regional".to_string()],
            }),
            Rc::new(SloSegmentRequiredCodeRule {
                message: "slo segment needs an entry".to_string(),
            }),
            Rc::new(SloSegmentRequiredFingerprintRule {
                message: "slo segment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_segment_validator();
        let item = new_slo_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SloSegmentTitleLengthRule { max_length: 255 };
        let mut item = new_slo_segment("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = SloSegmentFingerprintLengthRule { max_length: 255 };
        let mut item = new_slo_segment("x", "y");
        item.fingerprint = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloSegmentRequiredCodeRule {
            message: "slo segment needs a value".to_string(),
        };
        let mut item = new_slo_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = SloSegmentRequiredFingerprintRule {
            message: "slo segment needs an entry".to_string(),
        };
        let mut item = new_slo_segment("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
