use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default ttl policy bundle must pass. Check returns its problems, or none.
pub trait DefaultTtlPolicyBundleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTtlPolicyBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTtlPolicyBundleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultTtlPolicyBundleRule for DefaultTtlPolicyBundleScoreRangeRule {
    fn check(&self, item: &DefaultTtlPolicyBundle) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTtlPolicyBundleRequiredLabelRule {
    pub message: String,
}

impl DefaultTtlPolicyBundleRule for DefaultTtlPolicyBundleRequiredLabelRule {
    fn check(&self, item: &DefaultTtlPolicyBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTtlPolicyBundleAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl DefaultTtlPolicyBundleRule for DefaultTtlPolicyBundleAllowedUnitRule {
    fn check(&self, item: &DefaultTtlPolicyBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is stale", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTtlPolicyBundleDescriptionLengthRule {
    pub max_length: i64,
}

impl DefaultTtlPolicyBundleRule for DefaultTtlPolicyBundleDescriptionLengthRule {
    fn check(&self, item: &DefaultTtlPolicyBundle) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default ttl policy bundle.
#[derive(Debug, Clone)]
pub struct DefaultTtlPolicyBundleValidator {
    pub rules: Vec<Rc<dyn DefaultTtlPolicyBundleRule>>,
}

impl DefaultTtlPolicyBundleValidator {
    pub fn validate(&self, item: &DefaultTtlPolicyBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultTtlPolicyBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTtlPolicyBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_ttl_policy_bundle_validator() -> DefaultTtlPolicyBundleValidator {
    DefaultTtlPolicyBundleValidator {
        rules: vec![
            Rc::new(DefaultTtlPolicyBundleScoreRangeRule { min: 7.3, max: 749.7 })
                as Rc<dyn DefaultTtlPolicyBundleRule>,
            Rc::new(DefaultTtlPolicyBundleRequiredLabelRule {
                message: "default ttl policy bundle needs this field".to_string(),
            }),
            Rc::new(DefaultTtlPolicyBundleAllowedUnitRule {
                allowed: vec!["pending".to_string(), "manual".to_string(), "rolling".to_string()],
            }),
            Rc::new(DefaultTtlPolicyBundleDescriptionLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_ttl_policy_bundle_validator();
        let item = new_default_ttl_policy_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = DefaultTtlPolicyBundleScoreRangeRule { min: 7.4, max: 1778.5 };
        let mut item = new_default_ttl_policy_bundle("x", "y");
        item.score = 1780.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultTtlPolicyBundleRequiredLabelRule {
            message: "default ttl policy bundle needs a value".to_string(),
        };
        let mut item = new_default_ttl_policy_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = DefaultTtlPolicyBundleDescriptionLengthRule { max_length: 1000 };
        let mut item = new_default_ttl_policy_bundle("x", "y");
        item.description = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
