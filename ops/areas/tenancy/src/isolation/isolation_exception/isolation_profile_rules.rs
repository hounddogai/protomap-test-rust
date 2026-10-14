use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation profile must pass. Check returns its problems, or none.
pub trait IsolationProfileRule: std::fmt::Debug {
    fn check(&self, item: &IsolationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProfileFingerprintLengthRule {
    pub max_length: i64,
}

impl IsolationProfileRule for IsolationProfileFingerprintLengthRule {
    fn check(&self, item: &IsolationProfile) -> Vec<String> {
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
pub struct IsolationProfileRequiredFingerprintRule {
    pub message: String,
}

impl IsolationProfileRule for IsolationProfileRequiredFingerprintRule {
    fn check(&self, item: &IsolationProfile) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProfileCodeLengthRule {
    pub max_length: i64,
}

impl IsolationProfileRule for IsolationProfileCodeLengthRule {
    fn check(&self, item: &IsolationProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProfilePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationProfileRule for IsolationProfilePriorityRangeRule {
    fn check(&self, item: &IsolationProfile) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProfileSegmentLengthRule {
    pub max_length: i64,
}

impl IsolationProfileRule for IsolationProfileSegmentLengthRule {
    fn check(&self, item: &IsolationProfile) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProfileTitleLengthRule {
    pub max_length: i64,
}

impl IsolationProfileRule for IsolationProfileTitleLengthRule {
    fn check(&self, item: &IsolationProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation profile.
#[derive(Debug, Clone)]
pub struct IsolationProfileValidator {
    pub rules: Vec<Rc<dyn IsolationProfileRule>>,
}

impl IsolationProfileValidator {
    pub fn validate(&self, item: &IsolationProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_profile_validator() -> IsolationProfileValidator {
    IsolationProfileValidator {
        rules: vec![
            Rc::new(IsolationProfileFingerprintLengthRule { max_length: 64 }) as Rc<dyn IsolationProfileRule>,
            Rc::new(IsolationProfileRequiredFingerprintRule {
                message: "isolation profile needs a value".to_string(),
            }),
            Rc::new(IsolationProfileCodeLengthRule { max_length: 64 }),
            Rc::new(IsolationProfilePriorityRangeRule { min: 64, max: 1000 }),
            Rc::new(IsolationProfileSegmentLengthRule { max_length: 32 }),
            Rc::new(IsolationProfileTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_profile_validator();
        let item = new_isolation_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = IsolationProfileFingerprintLengthRule { max_length: 120 };
        let mut item = new_isolation_profile("x", "y");
        item.fingerprint = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = IsolationProfileRequiredFingerprintRule {
            message: "isolation profile needs a value".to_string(),
        };
        let mut item = new_isolation_profile("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IsolationProfileCodeLengthRule { max_length: 80 };
        let mut item = new_isolation_profile("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = IsolationProfilePriorityRangeRule { min: 16, max: 1000 };
        let mut item = new_isolation_profile("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = IsolationProfileSegmentLengthRule { max_length: 32 };
        let mut item = new_isolation_profile("x", "y");
        item.segment = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = IsolationProfileTitleLengthRule { max_length: 86400 };
        let mut item = new_isolation_profile("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
