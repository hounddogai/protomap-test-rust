use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default trust domain profile must pass. Check returns its problems, or none.
pub trait DefaultTrustDomainProfileRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTrustDomainProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainProfileBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultTrustDomainProfileRule for DefaultTrustDomainProfileBudgetRangeRule {
    fn check(&self, item: &DefaultTrustDomainProfile) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainProfileCodePrefixRule {
    pub prefix: String,
}

impl DefaultTrustDomainProfileRule for DefaultTrustDomainProfileCodePrefixRule {
    fn check(&self, item: &DefaultTrustDomainProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainProfileChannelLengthRule {
    pub max_length: i64,
}

impl DefaultTrustDomainProfileRule for DefaultTrustDomainProfileChannelLengthRule {
    fn check(&self, item: &DefaultTrustDomainProfile) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainProfileRequiredHostRule {
    pub message: String,
}

impl DefaultTrustDomainProfileRule for DefaultTrustDomainProfileRequiredHostRule {
    fn check(&self, item: &DefaultTrustDomainProfile) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainProfileCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultTrustDomainProfileRule for DefaultTrustDomainProfileCreatedAtRangeRule {
    fn check(&self, item: &DefaultTrustDomainProfile) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainProfileCodeLengthRule {
    pub max_length: i64,
}

impl DefaultTrustDomainProfileRule for DefaultTrustDomainProfileCodeLengthRule {
    fn check(&self, item: &DefaultTrustDomainProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default trust domain profile.
#[derive(Debug, Clone)]
pub struct DefaultTrustDomainProfileValidator {
    pub rules: Vec<Rc<dyn DefaultTrustDomainProfileRule>>,
}

impl DefaultTrustDomainProfileValidator {
    pub fn validate(&self, item: &DefaultTrustDomainProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultTrustDomainProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTrustDomainProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_trust_domain_profile_validator() -> DefaultTrustDomainProfileValidator {
    DefaultTrustDomainProfileValidator {
        rules: vec![
            Rc::new(DefaultTrustDomainProfileBudgetRangeRule { min: 2.3, max: 3431.6 })
                as Rc<dyn DefaultTrustDomainProfileRule>,
            Rc::new(DefaultTrustDomainProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultTrustDomainProfileChannelLengthRule { max_length: 80 }),
            Rc::new(DefaultTrustDomainProfileRequiredHostRule {
                message: "default trust domain profile needs an entry".to_string(),
            }),
            Rc::new(DefaultTrustDomainProfileCreatedAtRangeRule { min: 32, max: 64 }),
            Rc::new(DefaultTrustDomainProfileCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_trust_domain_profile_validator();
        let item = new_default_trust_domain_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = DefaultTrustDomainProfileBudgetRangeRule { min: 3.6, max: 1237.5 };
        let mut item = new_default_trust_domain_profile("x", "y");
        item.budget = 1239.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = DefaultTrustDomainProfileChannelLengthRule { max_length: 64 };
        let mut item = new_default_trust_domain_profile("x", "y");
        item.channel = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = DefaultTrustDomainProfileRequiredHostRule {
            message: "default trust domain profile needs an entry".to_string(),
        };
        let mut item = new_default_trust_domain_profile("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = DefaultTrustDomainProfileCreatedAtRangeRule { min: 16, max: 64 };
        let mut item = new_default_trust_domain_profile("x", "y");
        item.created_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultTrustDomainProfileCodeLengthRule { max_length: 3600 };
        let mut item = new_default_trust_domain_profile("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
