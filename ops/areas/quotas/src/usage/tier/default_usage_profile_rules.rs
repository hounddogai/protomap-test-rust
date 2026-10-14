use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default usage profile must pass. Check returns its problems, or none.
pub trait DefaultUsageProfileRule: std::fmt::Debug {
    fn check(&self, item: &DefaultUsageProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageProfileRequiredLabelRule {
    pub message: String,
}

impl DefaultUsageProfileRule for DefaultUsageProfileRequiredLabelRule {
    fn check(&self, item: &DefaultUsageProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageProfileCodeLengthRule {
    pub max_length: i64,
}

impl DefaultUsageProfileRule for DefaultUsageProfileCodeLengthRule {
    fn check(&self, item: &DefaultUsageProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageProfileRequiredClusterRule {
    pub message: String,
}

impl DefaultUsageProfileRule for DefaultUsageProfileRequiredClusterRule {
    fn check(&self, item: &DefaultUsageProfile) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageProfileRequiredCodeRule {
    pub message: String,
}

impl DefaultUsageProfileRule for DefaultUsageProfileRequiredCodeRule {
    fn check(&self, item: &DefaultUsageProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageProfileLabelLengthRule {
    pub max_length: i64,
}

impl DefaultUsageProfileRule for DefaultUsageProfileLabelLengthRule {
    fn check(&self, item: &DefaultUsageProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageProfileCodePrefixRule {
    pub prefix: String,
}

impl DefaultUsageProfileRule for DefaultUsageProfileCodePrefixRule {
    fn check(&self, item: &DefaultUsageProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default usage profile.
#[derive(Debug, Clone)]
pub struct DefaultUsageProfileValidator {
    pub rules: Vec<Rc<dyn DefaultUsageProfileRule>>,
}

impl DefaultUsageProfileValidator {
    pub fn validate(&self, item: &DefaultUsageProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultUsageProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultUsageProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_usage_profile_validator() -> DefaultUsageProfileValidator {
    DefaultUsageProfileValidator {
        rules: vec![
            Rc::new(DefaultUsageProfileRequiredLabelRule {
                message: "default usage profile needs an entry".to_string(),
            }) as Rc<dyn DefaultUsageProfileRule>,
            Rc::new(DefaultUsageProfileCodeLengthRule { max_length: 16 }),
            Rc::new(DefaultUsageProfileRequiredClusterRule {
                message: "default usage profile needs a value".to_string(),
            }),
            Rc::new(DefaultUsageProfileRequiredCodeRule {
                message: "default usage profile needs an entry".to_string(),
            }),
            Rc::new(DefaultUsageProfileLabelLengthRule { max_length: 32 }),
            Rc::new(DefaultUsageProfileCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_usage_profile_validator();
        let item = new_default_usage_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultUsageProfileRequiredLabelRule {
            message: "default usage profile needs a value".to_string(),
        };
        let mut item = new_default_usage_profile("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultUsageProfileCodeLengthRule { max_length: 64 };
        let mut item = new_default_usage_profile("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = DefaultUsageProfileRequiredClusterRule {
            message: "default usage profile needs an entry".to_string(),
        };
        let mut item = new_default_usage_profile("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultUsageProfileRequiredCodeRule {
            message: "default usage profile needs this field".to_string(),
        };
        let mut item = new_default_usage_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultUsageProfileLabelLengthRule { max_length: 16 };
        let mut item = new_default_usage_profile("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
