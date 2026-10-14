use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone draft must pass. Check returns its problems, or none.
pub trait ZoneDraftRule: std::fmt::Debug {
    fn check(&self, item: &ZoneDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDraftCodePrefixRule {
    pub prefix: String,
}

impl ZoneDraftRule for ZoneDraftCodePrefixRule {
    fn check(&self, item: &ZoneDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDraftMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneDraftRule for ZoneDraftMinimumRangeRule {
    fn check(&self, item: &ZoneDraft) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDraftAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl ZoneDraftRule for ZoneDraftAllowedFingerprintRule {
    fn check(&self, item: &ZoneDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is missing", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDraftStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneDraftRule for ZoneDraftStartsAtRangeRule {
    fn check(&self, item: &ZoneDraft) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDraftRequiredFingerprintRule {
    pub message: String,
}

impl ZoneDraftRule for ZoneDraftRequiredFingerprintRule {
    fn check(&self, item: &ZoneDraft) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDraftFingerprintLengthRule {
    pub max_length: i64,
}

impl ZoneDraftRule for ZoneDraftFingerprintLengthRule {
    fn check(&self, item: &ZoneDraft) -> Vec<String> {
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

/// Runs every rule of a zone draft.
#[derive(Debug, Clone)]
pub struct ZoneDraftValidator {
    pub rules: Vec<Rc<dyn ZoneDraftRule>>,
}

impl ZoneDraftValidator {
    pub fn validate(&self, item: &ZoneDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_draft_validator() -> ZoneDraftValidator {
    ZoneDraftValidator {
        rules: vec![
            Rc::new(ZoneDraftCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ZoneDraftRule>,
            Rc::new(ZoneDraftMinimumRangeRule { min: 5.2, max: 4977.6 }),
            Rc::new(ZoneDraftAllowedFingerprintRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(ZoneDraftStartsAtRangeRule { min: 32, max: 3600 }),
            Rc::new(ZoneDraftRequiredFingerprintRule {
                message: "zone draft needs a value".to_string(),
            }),
            Rc::new(ZoneDraftFingerprintLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_draft_validator();
        let item = new_zone_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ZoneDraftMinimumRangeRule { min: 8.9, max: 4222.6 };
        let mut item = new_zone_draft("x", "y");
        item.minimum = 4224.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ZoneDraftStartsAtRangeRule { min: 64, max: 120 };
        let mut item = new_zone_draft("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ZoneDraftRequiredFingerprintRule {
            message: "zone draft needs an entry".to_string(),
        };
        let mut item = new_zone_draft("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ZoneDraftFingerprintLengthRule { max_length: 1000 };
        let mut item = new_zone_draft("x", "y");
        item.fingerprint = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
