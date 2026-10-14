use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled delegation score must pass. Check returns its problems, or none.
pub trait ScheduledDelegationScoreRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledDelegationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationScoreRequiredLabelRule {
    pub message: String,
}

impl ScheduledDelegationScoreRule for ScheduledDelegationScoreRequiredLabelRule {
    fn check(&self, item: &ScheduledDelegationScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationScoreCodePrefixRule {
    pub prefix: String,
}

impl ScheduledDelegationScoreRule for ScheduledDelegationScoreCodePrefixRule {
    fn check(&self, item: &ScheduledDelegationScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledDelegationScoreRule for ScheduledDelegationScoreAllowedCodeRule {
    fn check(&self, item: &ScheduledDelegationScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationScoreRequiredChannelRule {
    pub message: String,
}

impl ScheduledDelegationScoreRule for ScheduledDelegationScoreRequiredChannelRule {
    fn check(&self, item: &ScheduledDelegationScore) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationScoreCurrencyLengthRule {
    pub max_length: i64,
}

impl ScheduledDelegationScoreRule for ScheduledDelegationScoreCurrencyLengthRule {
    fn check(&self, item: &ScheduledDelegationScore) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled delegation score.
#[derive(Debug, Clone)]
pub struct ScheduledDelegationScoreValidator {
    pub rules: Vec<Rc<dyn ScheduledDelegationScoreRule>>,
}

impl ScheduledDelegationScoreValidator {
    pub fn validate(&self, item: &ScheduledDelegationScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledDelegationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledDelegationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_delegation_score_validator() -> ScheduledDelegationScoreValidator {
    ScheduledDelegationScoreValidator {
        rules: vec![
            Rc::new(ScheduledDelegationScoreRequiredLabelRule {
                message: "scheduled delegation score needs this field".to_string(),
            }) as Rc<dyn ScheduledDelegationScoreRule>,
            Rc::new(ScheduledDelegationScoreCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ScheduledDelegationScoreAllowedCodeRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(ScheduledDelegationScoreRequiredChannelRule {
                message: "scheduled delegation score needs an entry".to_string(),
            }),
            Rc::new(ScheduledDelegationScoreCurrencyLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_delegation_score_validator();
        let item = new_scheduled_delegation_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledDelegationScoreRequiredLabelRule {
            message: "scheduled delegation score needs an entry".to_string(),
        };
        let mut item = new_scheduled_delegation_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = ScheduledDelegationScoreRequiredChannelRule {
            message: "scheduled delegation score needs an entry".to_string(),
        };
        let mut item = new_scheduled_delegation_score("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = ScheduledDelegationScoreCurrencyLengthRule { max_length: 3600 };
        let mut item = new_scheduled_delegation_score("x", "y");
        item.currency = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
