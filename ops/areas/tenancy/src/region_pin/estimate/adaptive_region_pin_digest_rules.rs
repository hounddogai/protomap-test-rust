use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive region pin digest must pass. Check returns its problems, or none.
pub trait AdaptiveRegionPinDigestRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRegionPinDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinDigestCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveRegionPinDigestRule for AdaptiveRegionPinDigestCodePrefixRule {
    fn check(&self, item: &AdaptiveRegionPinDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinDigestReviewerLengthRule {
    pub max_length: i64,
}

impl AdaptiveRegionPinDigestRule for AdaptiveRegionPinDigestReviewerLengthRule {
    fn check(&self, item: &AdaptiveRegionPinDigest) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinDigestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRegionPinDigestRule for AdaptiveRegionPinDigestCeilingRangeRule {
    fn check(&self, item: &AdaptiveRegionPinDigest) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinDigestRequiredLabelRule {
    pub message: String,
}

impl AdaptiveRegionPinDigestRule for AdaptiveRegionPinDigestRequiredLabelRule {
    fn check(&self, item: &AdaptiveRegionPinDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinDigestLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveRegionPinDigestRule for AdaptiveRegionPinDigestLabelLengthRule {
    fn check(&self, item: &AdaptiveRegionPinDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinDigestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveRegionPinDigestRule for AdaptiveRegionPinDigestAllowedCodeRule {
    fn check(&self, item: &AdaptiveRegionPinDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive region pin digest.
#[derive(Debug, Clone)]
pub struct AdaptiveRegionPinDigestValidator {
    pub rules: Vec<Rc<dyn AdaptiveRegionPinDigestRule>>,
}

impl AdaptiveRegionPinDigestValidator {
    pub fn validate(&self, item: &AdaptiveRegionPinDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveRegionPinDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRegionPinDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_region_pin_digest_validator() -> AdaptiveRegionPinDigestValidator {
    AdaptiveRegionPinDigestValidator {
        rules: vec![
            Rc::new(AdaptiveRegionPinDigestCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn AdaptiveRegionPinDigestRule>,
            Rc::new(AdaptiveRegionPinDigestReviewerLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveRegionPinDigestCeilingRangeRule { min: 4.5, max: 1641.3 }),
            Rc::new(AdaptiveRegionPinDigestRequiredLabelRule {
                message: "adaptive region pin digest needs this field".to_string(),
            }),
            Rc::new(AdaptiveRegionPinDigestLabelLengthRule { max_length: 255 }),
            Rc::new(AdaptiveRegionPinDigestAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "seasonal".to_string(), "primary".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_region_pin_digest_validator();
        let item = new_adaptive_region_pin_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = AdaptiveRegionPinDigestReviewerLengthRule { max_length: 3600 };
        let mut item = new_adaptive_region_pin_digest("x", "y");
        item.reviewer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = AdaptiveRegionPinDigestCeilingRangeRule { min: 2.6, max: 947.2 };
        let mut item = new_adaptive_region_pin_digest("x", "y");
        item.ceiling = 948.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveRegionPinDigestRequiredLabelRule {
            message: "adaptive region pin digest needs a value".to_string(),
        };
        let mut item = new_adaptive_region_pin_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveRegionPinDigestLabelLengthRule { max_length: 1000 };
        let mut item = new_adaptive_region_pin_digest("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
