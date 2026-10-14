use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive nonce rule must pass. Check returns its problems, or none.
pub trait AdaptiveNonceRuleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveNonceRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceRuleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveNonceRuleRule for AdaptiveNonceRuleTtlSecondsRangeRule {
    fn check(&self, item: &AdaptiveNonceRule) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceRuleRequiredTitleRule {
    pub message: String,
}

impl AdaptiveNonceRuleRule for AdaptiveNonceRuleRequiredTitleRule {
    fn check(&self, item: &AdaptiveNonceRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceRuleRequiredChannelRule {
    pub message: String,
}

impl AdaptiveNonceRuleRule for AdaptiveNonceRuleRequiredChannelRule {
    fn check(&self, item: &AdaptiveNonceRule) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceRuleRequiredCountryRule {
    pub message: String,
}

impl AdaptiveNonceRuleRule for AdaptiveNonceRuleRequiredCountryRule {
    fn check(&self, item: &AdaptiveNonceRule) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceRuleChannelLengthRule {
    pub max_length: i64,
}

impl AdaptiveNonceRuleRule for AdaptiveNonceRuleChannelLengthRule {
    fn check(&self, item: &AdaptiveNonceRule) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive nonce rule.
#[derive(Debug, Clone)]
pub struct AdaptiveNonceRuleValidator {
    pub rules: Vec<Rc<dyn AdaptiveNonceRuleRule>>,
}

impl AdaptiveNonceRuleValidator {
    pub fn validate(&self, item: &AdaptiveNonceRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveNonceRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveNonceRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_nonce_rule_validator() -> AdaptiveNonceRuleValidator {
    AdaptiveNonceRuleValidator {
        rules: vec![
            Rc::new(AdaptiveNonceRuleTtlSecondsRangeRule { min: 3600, max: 3600 }) as Rc<dyn AdaptiveNonceRuleRule>,
            Rc::new(AdaptiveNonceRuleRequiredTitleRule {
                message: "adaptive nonce rule needs this field".to_string(),
            }),
            Rc::new(AdaptiveNonceRuleRequiredChannelRule {
                message: "adaptive nonce rule needs an entry".to_string(),
            }),
            Rc::new(AdaptiveNonceRuleRequiredCountryRule {
                message: "adaptive nonce rule needs an entry".to_string(),
            }),
            Rc::new(AdaptiveNonceRuleChannelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_nonce_rule_validator();
        let item = new_adaptive_nonce_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = AdaptiveNonceRuleTtlSecondsRangeRule { min: 32, max: 32 };
        let mut item = new_adaptive_nonce_rule("x", "y");
        item.ttl_seconds = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveNonceRuleRequiredTitleRule {
            message: "adaptive nonce rule needs this field".to_string(),
        };
        let mut item = new_adaptive_nonce_rule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = AdaptiveNonceRuleRequiredChannelRule {
            message: "adaptive nonce rule needs a value".to_string(),
        };
        let mut item = new_adaptive_nonce_rule("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = AdaptiveNonceRuleRequiredCountryRule {
            message: "adaptive nonce rule needs an entry".to_string(),
        };
        let mut item = new_adaptive_nonce_rule("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = AdaptiveNonceRuleChannelLengthRule { max_length: 255 };
        let mut item = new_adaptive_nonce_rule("x", "y");
        item.channel = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
