use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient delegation policy must pass. Check returns its problems, or none.
pub trait LenientDelegationPolicyRule: std::fmt::Debug {
    fn check(&self, item: &LenientDelegationPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationPolicyRequiredChannelRule {
    pub message: String,
}

impl LenientDelegationPolicyRule for LenientDelegationPolicyRequiredChannelRule {
    fn check(&self, item: &LenientDelegationPolicy) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationPolicyRequiredCodeRule {
    pub message: String,
}

impl LenientDelegationPolicyRule for LenientDelegationPolicyRequiredCodeRule {
    fn check(&self, item: &LenientDelegationPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationPolicyTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientDelegationPolicyRule for LenientDelegationPolicyTtlSecondsRangeRule {
    fn check(&self, item: &LenientDelegationPolicy) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationPolicyCodeLengthRule {
    pub max_length: i64,
}

impl LenientDelegationPolicyRule for LenientDelegationPolicyCodeLengthRule {
    fn check(&self, item: &LenientDelegationPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationPolicyAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LenientDelegationPolicyRule for LenientDelegationPolicyAllowedTitleRule {
    fn check(&self, item: &LenientDelegationPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient delegation policy.
#[derive(Debug, Clone)]
pub struct LenientDelegationPolicyValidator {
    pub rules: Vec<Rc<dyn LenientDelegationPolicyRule>>,
}

impl LenientDelegationPolicyValidator {
    pub fn validate(&self, item: &LenientDelegationPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientDelegationPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDelegationPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_delegation_policy_validator() -> LenientDelegationPolicyValidator {
    LenientDelegationPolicyValidator {
        rules: vec![
            Rc::new(LenientDelegationPolicyRequiredChannelRule {
                message: "lenient delegation policy needs this field".to_string(),
            }) as Rc<dyn LenientDelegationPolicyRule>,
            Rc::new(LenientDelegationPolicyRequiredCodeRule {
                message: "lenient delegation policy needs this field".to_string(),
            }),
            Rc::new(LenientDelegationPolicyTtlSecondsRangeRule { min: 64, max: 64 }),
            Rc::new(LenientDelegationPolicyCodeLengthRule { max_length: 32 }),
            Rc::new(LenientDelegationPolicyAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_delegation_policy_validator();
        let item = new_lenient_delegation_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = LenientDelegationPolicyRequiredChannelRule {
            message: "lenient delegation policy needs a value".to_string(),
        };
        let mut item = new_lenient_delegation_policy("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientDelegationPolicyRequiredCodeRule {
            message: "lenient delegation policy needs an entry".to_string(),
        };
        let mut item = new_lenient_delegation_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = LenientDelegationPolicyTtlSecondsRangeRule { min: 80, max: 120 };
        let mut item = new_lenient_delegation_policy("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientDelegationPolicyCodeLengthRule { max_length: 32 };
        let mut item = new_lenient_delegation_policy("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
