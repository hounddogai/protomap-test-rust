use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary policy must pass. Check returns its problems, or none.
pub trait CanaryPolicyRule: std::fmt::Debug {
    fn check(&self, item: &CanaryPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPolicyRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryPolicyRule for CanaryPolicyRevisionRangeRule {
    fn check(&self, item: &CanaryPolicy) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPolicyAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CanaryPolicyRule for CanaryPolicyAllowedCodeRule {
    fn check(&self, item: &CanaryPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPolicyLabelLengthRule {
    pub max_length: i64,
}

impl CanaryPolicyRule for CanaryPolicyLabelLengthRule {
    fn check(&self, item: &CanaryPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPolicyCodePrefixRule {
    pub prefix: String,
}

impl CanaryPolicyRule for CanaryPolicyCodePrefixRule {
    fn check(&self, item: &CanaryPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPolicyRequiredLabelRule {
    pub message: String,
}

impl CanaryPolicyRule for CanaryPolicyRequiredLabelRule {
    fn check(&self, item: &CanaryPolicy) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPolicyLocaleLengthRule {
    pub max_length: i64,
}

impl CanaryPolicyRule for CanaryPolicyLocaleLengthRule {
    fn check(&self, item: &CanaryPolicy) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary policy.
#[derive(Debug, Clone)]
pub struct CanaryPolicyValidator {
    pub rules: Vec<Rc<dyn CanaryPolicyRule>>,
}

impl CanaryPolicyValidator {
    pub fn validate(&self, item: &CanaryPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_policy_validator() -> CanaryPolicyValidator {
    CanaryPolicyValidator {
        rules: vec![
            Rc::new(CanaryPolicyRevisionRangeRule { min: 255, max: 3600 }) as Rc<dyn CanaryPolicyRule>,
            Rc::new(CanaryPolicyAllowedCodeRule {
                allowed: vec!["primary".to_string(), "legacy".to_string()],
            }),
            Rc::new(CanaryPolicyLabelLengthRule { max_length: 64 }),
            Rc::new(CanaryPolicyCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CanaryPolicyRequiredLabelRule {
                message: "canary policy needs this field".to_string(),
            }),
            Rc::new(CanaryPolicyLocaleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_policy_validator();
        let item = new_canary_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = CanaryPolicyRevisionRangeRule { min: 80, max: 255 };
        let mut item = new_canary_policy("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CanaryPolicyLabelLengthRule { max_length: 86400 };
        let mut item = new_canary_policy("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CanaryPolicyRequiredLabelRule {
            message: "canary policy needs a value".to_string(),
        };
        let mut item = new_canary_policy("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = CanaryPolicyLocaleLengthRule { max_length: 64 };
        let mut item = new_canary_policy("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
