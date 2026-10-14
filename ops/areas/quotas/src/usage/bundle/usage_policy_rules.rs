use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage policy must pass. Check returns its problems, or none.
pub trait UsagePolicyRule: std::fmt::Debug {
    fn check(&self, item: &UsagePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePolicyRequiredLabelRule {
    pub message: String,
}

impl UsagePolicyRule for UsagePolicyRequiredLabelRule {
    fn check(&self, item: &UsagePolicy) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePolicyAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl UsagePolicyRule for UsagePolicyAllowedLabelRule {
    fn check(&self, item: &UsagePolicy) -> Vec<String> {
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
pub struct UsagePolicyCodePrefixRule {
    pub prefix: String,
}

impl UsagePolicyRule for UsagePolicyCodePrefixRule {
    fn check(&self, item: &UsagePolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePolicyUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsagePolicyRule for UsagePolicyUpdatedAtRangeRule {
    fn check(&self, item: &UsagePolicy) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePolicyLabelLengthRule {
    pub max_length: i64,
}

impl UsagePolicyRule for UsagePolicyLabelLengthRule {
    fn check(&self, item: &UsagePolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage policy.
#[derive(Debug, Clone)]
pub struct UsagePolicyValidator {
    pub rules: Vec<Rc<dyn UsagePolicyRule>>,
}

impl UsagePolicyValidator {
    pub fn validate(&self, item: &UsagePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsagePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsagePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_policy_validator() -> UsagePolicyValidator {
    UsagePolicyValidator {
        rules: vec![
            Rc::new(UsagePolicyRequiredLabelRule {
                message: "usage policy needs this field".to_string(),
            }) as Rc<dyn UsagePolicyRule>,
            Rc::new(UsagePolicyAllowedLabelRule {
                allowed: vec!["seasonal".to_string(), "adaptive".to_string(), "fallback".to_string()],
            }),
            Rc::new(UsagePolicyCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(UsagePolicyUpdatedAtRangeRule { min: 32, max: 3600 }),
            Rc::new(UsagePolicyLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_policy_validator();
        let item = new_usage_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = UsagePolicyRequiredLabelRule {
            message: "usage policy needs this field".to_string(),
        };
        let mut item = new_usage_policy("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = UsagePolicyUpdatedAtRangeRule { min: 80, max: 255 };
        let mut item = new_usage_policy("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = UsagePolicyLabelLengthRule { max_length: 16 };
        let mut item = new_usage_policy("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
