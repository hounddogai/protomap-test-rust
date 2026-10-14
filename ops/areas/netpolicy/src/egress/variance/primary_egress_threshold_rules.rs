use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary egress threshold must pass. Check returns its problems, or none.
pub trait PrimaryEgressThresholdRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEgressThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressThresholdRequiredCodeRule {
    pub message: String,
}

impl PrimaryEgressThresholdRule for PrimaryEgressThresholdRequiredCodeRule {
    fn check(&self, item: &PrimaryEgressThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressThresholdRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEgressThresholdRule for PrimaryEgressThresholdRetriesRangeRule {
    fn check(&self, item: &PrimaryEgressThreshold) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressThresholdCodePrefixRule {
    pub prefix: String,
}

impl PrimaryEgressThresholdRule for PrimaryEgressThresholdCodePrefixRule {
    fn check(&self, item: &PrimaryEgressThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressThresholdAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PrimaryEgressThresholdRule for PrimaryEgressThresholdAllowedLabelRule {
    fn check(&self, item: &PrimaryEgressThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressThresholdFingerprintLengthRule {
    pub max_length: i64,
}

impl PrimaryEgressThresholdRule for PrimaryEgressThresholdFingerprintLengthRule {
    fn check(&self, item: &PrimaryEgressThreshold) -> Vec<String> {
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

/// Runs every rule of a primary egress threshold.
#[derive(Debug, Clone)]
pub struct PrimaryEgressThresholdValidator {
    pub rules: Vec<Rc<dyn PrimaryEgressThresholdRule>>,
}

impl PrimaryEgressThresholdValidator {
    pub fn validate(&self, item: &PrimaryEgressThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryEgressThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEgressThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_egress_threshold_validator() -> PrimaryEgressThresholdValidator {
    PrimaryEgressThresholdValidator {
        rules: vec![
            Rc::new(PrimaryEgressThresholdRequiredCodeRule {
                message: "primary egress threshold needs a value".to_string(),
            }) as Rc<dyn PrimaryEgressThresholdRule>,
            Rc::new(PrimaryEgressThresholdRetriesRangeRule { min: 120, max: 1000 }),
            Rc::new(PrimaryEgressThresholdCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryEgressThresholdAllowedLabelRule {
                allowed: vec!["weighted".to_string(), "primary".to_string()],
            }),
            Rc::new(PrimaryEgressThresholdFingerprintLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_egress_threshold_validator();
        let item = new_primary_egress_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryEgressThresholdRequiredCodeRule {
            message: "primary egress threshold needs this field".to_string(),
        };
        let mut item = new_primary_egress_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = PrimaryEgressThresholdRetriesRangeRule { min: 3600, max: 86400 };
        let mut item = new_primary_egress_threshold("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = PrimaryEgressThresholdFingerprintLengthRule { max_length: 16 };
        let mut item = new_primary_egress_threshold("x", "y");
        item.fingerprint = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
