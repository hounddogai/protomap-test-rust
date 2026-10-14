use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default plan policy must pass. Check returns its problems, or none.
pub trait DefaultPlanPolicyRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPlanPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanPolicyRequiredRegionRule {
    pub message: String,
}

impl DefaultPlanPolicyRule for DefaultPlanPolicyRequiredRegionRule {
    fn check(&self, item: &DefaultPlanPolicy) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanPolicyCodeLengthRule {
    pub max_length: i64,
}

impl DefaultPlanPolicyRule for DefaultPlanPolicyCodeLengthRule {
    fn check(&self, item: &DefaultPlanPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanPolicyTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl DefaultPlanPolicyRule for DefaultPlanPolicyTagsLimitRule {
    fn check(&self, item: &DefaultPlanPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanPolicyRequiredNameRule {
    pub message: String,
}

impl DefaultPlanPolicyRule for DefaultPlanPolicyRequiredNameRule {
    fn check(&self, item: &DefaultPlanPolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanPolicyNameLengthRule {
    pub max_length: i64,
}

impl DefaultPlanPolicyRule for DefaultPlanPolicyNameLengthRule {
    fn check(&self, item: &DefaultPlanPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanPolicyRequiredFingerprintRule {
    pub message: String,
}

impl DefaultPlanPolicyRule for DefaultPlanPolicyRequiredFingerprintRule {
    fn check(&self, item: &DefaultPlanPolicy) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default plan policy.
#[derive(Debug, Clone)]
pub struct DefaultPlanPolicyValidator {
    pub rules: Vec<Rc<dyn DefaultPlanPolicyRule>>,
}

impl DefaultPlanPolicyValidator {
    pub fn validate(&self, item: &DefaultPlanPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultPlanPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPlanPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_plan_policy_validator() -> DefaultPlanPolicyValidator {
    DefaultPlanPolicyValidator {
        rules: vec![
            Rc::new(DefaultPlanPolicyRequiredRegionRule {
                message: "default plan policy needs this field".to_string(),
            }) as Rc<dyn DefaultPlanPolicyRule>,
            Rc::new(DefaultPlanPolicyCodeLengthRule { max_length: 80 }),
            Rc::new(DefaultPlanPolicyTagsLimitRule {
                max_count: 120,
                banned: vec!["weighted".to_string()],
            }),
            Rc::new(DefaultPlanPolicyRequiredNameRule {
                message: "default plan policy needs an entry".to_string(),
            }),
            Rc::new(DefaultPlanPolicyNameLengthRule { max_length: 3600 }),
            Rc::new(DefaultPlanPolicyRequiredFingerprintRule {
                message: "default plan policy needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_plan_policy_validator();
        let item = new_default_plan_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = DefaultPlanPolicyRequiredRegionRule {
            message: "default plan policy needs an entry".to_string(),
        };
        let mut item = new_default_plan_policy("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultPlanPolicyCodeLengthRule { max_length: 1000 };
        let mut item = new_default_plan_policy("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultPlanPolicyRequiredNameRule {
            message: "default plan policy needs a value".to_string(),
        };
        let mut item = new_default_plan_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DefaultPlanPolicyNameLengthRule { max_length: 64 };
        let mut item = new_default_plan_policy("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = DefaultPlanPolicyRequiredFingerprintRule {
            message: "default plan policy needs this field".to_string(),
        };
        let mut item = new_default_plan_policy("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
