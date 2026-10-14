use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy verifier estimate must pass. Check returns its problems, or none.
pub trait LegacyVerifierEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LegacyVerifierEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierEstimateThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyVerifierEstimateRule for LegacyVerifierEstimateThresholdRangeRule {
    fn check(&self, item: &LegacyVerifierEstimate) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierEstimateAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LegacyVerifierEstimateRule for LegacyVerifierEstimateAllowedLabelRule {
    fn check(&self, item: &LegacyVerifierEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierEstimateRequiredCodeRule {
    pub message: String,
}

impl LegacyVerifierEstimateRule for LegacyVerifierEstimateRequiredCodeRule {
    fn check(&self, item: &LegacyVerifierEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierEstimateCodePrefixRule {
    pub prefix: String,
}

impl LegacyVerifierEstimateRule for LegacyVerifierEstimateCodePrefixRule {
    fn check(&self, item: &LegacyVerifierEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierEstimateLabelLengthRule {
    pub max_length: i64,
}

impl LegacyVerifierEstimateRule for LegacyVerifierEstimateLabelLengthRule {
    fn check(&self, item: &LegacyVerifierEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierEstimateRequiredLabelRule {
    pub message: String,
}

impl LegacyVerifierEstimateRule for LegacyVerifierEstimateRequiredLabelRule {
    fn check(&self, item: &LegacyVerifierEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy verifier estimate.
#[derive(Debug, Clone)]
pub struct LegacyVerifierEstimateValidator {
    pub rules: Vec<Rc<dyn LegacyVerifierEstimateRule>>,
}

impl LegacyVerifierEstimateValidator {
    pub fn validate(&self, item: &LegacyVerifierEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyVerifierEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyVerifierEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_verifier_estimate_validator() -> LegacyVerifierEstimateValidator {
    LegacyVerifierEstimateValidator {
        rules: vec![
            Rc::new(LegacyVerifierEstimateThresholdRangeRule { min: 2.0, max: 4825.1 })
                as Rc<dyn LegacyVerifierEstimateRule>,
            Rc::new(LegacyVerifierEstimateAllowedLabelRule {
                allowed: vec!["tiered".to_string()],
            }),
            Rc::new(LegacyVerifierEstimateRequiredCodeRule {
                message: "legacy verifier estimate needs an entry".to_string(),
            }),
            Rc::new(LegacyVerifierEstimateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LegacyVerifierEstimateLabelLengthRule { max_length: 255 }),
            Rc::new(LegacyVerifierEstimateRequiredLabelRule {
                message: "legacy verifier estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_verifier_estimate_validator();
        let item = new_legacy_verifier_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LegacyVerifierEstimateThresholdRangeRule { min: 8.4, max: 1623.3 };
        let mut item = new_legacy_verifier_estimate("x", "y");
        item.threshold = 1624.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyVerifierEstimateRequiredCodeRule {
            message: "legacy verifier estimate needs this field".to_string(),
        };
        let mut item = new_legacy_verifier_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyVerifierEstimateLabelLengthRule { max_length: 80 };
        let mut item = new_legacy_verifier_estimate("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyVerifierEstimateRequiredLabelRule {
            message: "legacy verifier estimate needs an entry".to_string(),
        };
        let mut item = new_legacy_verifier_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
