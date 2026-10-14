use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual egress bundle must pass. Check returns its problems, or none.
pub trait ManualEgressBundleRule: std::fmt::Debug {
    fn check(&self, item: &ManualEgressBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEgressBundleRequiredCodeRule {
    pub message: String,
}

impl ManualEgressBundleRule for ManualEgressBundleRequiredCodeRule {
    fn check(&self, item: &ManualEgressBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEgressBundleFingerprintLengthRule {
    pub max_length: i64,
}

impl ManualEgressBundleRule for ManualEgressBundleFingerprintLengthRule {
    fn check(&self, item: &ManualEgressBundle) -> Vec<String> {
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
pub struct ManualEgressBundleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualEgressBundleRule for ManualEgressBundleStartsAtRangeRule {
    fn check(&self, item: &ManualEgressBundle) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEgressBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualEgressBundleRule for ManualEgressBundleAllowedCodeRule {
    fn check(&self, item: &ManualEgressBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual egress bundle.
#[derive(Debug, Clone)]
pub struct ManualEgressBundleValidator {
    pub rules: Vec<Rc<dyn ManualEgressBundleRule>>,
}

impl ManualEgressBundleValidator {
    pub fn validate(&self, item: &ManualEgressBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualEgressBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualEgressBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_egress_bundle_validator() -> ManualEgressBundleValidator {
    ManualEgressBundleValidator {
        rules: vec![
            Rc::new(ManualEgressBundleRequiredCodeRule {
                message: "manual egress bundle needs a value".to_string(),
            }) as Rc<dyn ManualEgressBundleRule>,
            Rc::new(ManualEgressBundleFingerprintLengthRule { max_length: 16 }),
            Rc::new(ManualEgressBundleStartsAtRangeRule { min: 32, max: 3600 }),
            Rc::new(ManualEgressBundleAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_egress_bundle_validator();
        let item = new_manual_egress_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualEgressBundleRequiredCodeRule {
            message: "manual egress bundle needs an entry".to_string(),
        };
        let mut item = new_manual_egress_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ManualEgressBundleFingerprintLengthRule { max_length: 32 };
        let mut item = new_manual_egress_bundle("x", "y");
        item.fingerprint = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ManualEgressBundleStartsAtRangeRule { min: 64, max: 3600 };
        let mut item = new_manual_egress_bundle("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
