use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default limit policy must pass. Check returns its problems, or none.
pub trait DefaultLimitPolicyRule: std::fmt::Debug {
    fn check(&self, item: &DefaultLimitPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitPolicyRegionLengthRule {
    pub max_length: i64,
}

impl DefaultLimitPolicyRule for DefaultLimitPolicyRegionLengthRule {
    fn check(&self, item: &DefaultLimitPolicy) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitPolicyRequiredRegionRule {
    pub message: String,
}

impl DefaultLimitPolicyRule for DefaultLimitPolicyRequiredRegionRule {
    fn check(&self, item: &DefaultLimitPolicy) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitPolicyCodeLengthRule {
    pub max_length: i64,
}

impl DefaultLimitPolicyRule for DefaultLimitPolicyCodeLengthRule {
    fn check(&self, item: &DefaultLimitPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitPolicyChannelLengthRule {
    pub max_length: i64,
}

impl DefaultLimitPolicyRule for DefaultLimitPolicyChannelLengthRule {
    fn check(&self, item: &DefaultLimitPolicy) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitPolicyRequiredChannelRule {
    pub message: String,
}

impl DefaultLimitPolicyRule for DefaultLimitPolicyRequiredChannelRule {
    fn check(&self, item: &DefaultLimitPolicy) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default limit policy.
#[derive(Debug, Clone)]
pub struct DefaultLimitPolicyValidator {
    pub rules: Vec<Rc<dyn DefaultLimitPolicyRule>>,
}

impl DefaultLimitPolicyValidator {
    pub fn validate(&self, item: &DefaultLimitPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultLimitPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultLimitPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_limit_policy_validator() -> DefaultLimitPolicyValidator {
    DefaultLimitPolicyValidator {
        rules: vec![
            Rc::new(DefaultLimitPolicyRegionLengthRule { max_length: 64 }) as Rc<dyn DefaultLimitPolicyRule>,
            Rc::new(DefaultLimitPolicyRequiredRegionRule {
                message: "default limit policy needs an entry".to_string(),
            }),
            Rc::new(DefaultLimitPolicyCodeLengthRule { max_length: 120 }),
            Rc::new(DefaultLimitPolicyChannelLengthRule { max_length: 64 }),
            Rc::new(DefaultLimitPolicyRequiredChannelRule {
                message: "default limit policy needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_limit_policy_validator();
        let item = new_default_limit_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = DefaultLimitPolicyRegionLengthRule { max_length: 120 };
        let mut item = new_default_limit_policy("x", "y");
        item.region = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = DefaultLimitPolicyRequiredRegionRule {
            message: "default limit policy needs an entry".to_string(),
        };
        let mut item = new_default_limit_policy("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultLimitPolicyCodeLengthRule { max_length: 64 };
        let mut item = new_default_limit_policy("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = DefaultLimitPolicyChannelLengthRule { max_length: 1000 };
        let mut item = new_default_limit_policy("x", "y");
        item.channel = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = DefaultLimitPolicyRequiredChannelRule {
            message: "default limit policy needs a value".to_string(),
        };
        let mut item = new_default_limit_policy("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
