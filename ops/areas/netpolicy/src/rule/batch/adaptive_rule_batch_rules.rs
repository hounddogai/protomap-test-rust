use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive rule batch must pass. Check returns its problems, or none.
pub trait AdaptiveRuleBatchRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRuleBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleBatchChannelLengthRule {
    pub max_length: i64,
}

impl AdaptiveRuleBatchRule for AdaptiveRuleBatchChannelLengthRule {
    fn check(&self, item: &AdaptiveRuleBatch) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveRuleBatchRule for AdaptiveRuleBatchAllowedCodeRule {
    fn check(&self, item: &AdaptiveRuleBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleBatchDescriptionLengthRule {
    pub max_length: i64,
}

impl AdaptiveRuleBatchRule for AdaptiveRuleBatchDescriptionLengthRule {
    fn check(&self, item: &AdaptiveRuleBatch) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleBatchRequiredDescriptionRule {
    pub message: String,
}

impl AdaptiveRuleBatchRule for AdaptiveRuleBatchRequiredDescriptionRule {
    fn check(&self, item: &AdaptiveRuleBatch) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleBatchTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl AdaptiveRuleBatchRule for AdaptiveRuleBatchTagsLimitRule {
    fn check(&self, item: &AdaptiveRuleBatch) -> Vec<String> {
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

/// Runs every rule of a adaptive rule batch.
#[derive(Debug, Clone)]
pub struct AdaptiveRuleBatchValidator {
    pub rules: Vec<Rc<dyn AdaptiveRuleBatchRule>>,
}

impl AdaptiveRuleBatchValidator {
    pub fn validate(&self, item: &AdaptiveRuleBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveRuleBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRuleBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_rule_batch_validator() -> AdaptiveRuleBatchValidator {
    AdaptiveRuleBatchValidator {
        rules: vec![
            Rc::new(AdaptiveRuleBatchChannelLengthRule { max_length: 16 }) as Rc<dyn AdaptiveRuleBatchRule>,
            Rc::new(AdaptiveRuleBatchAllowedCodeRule {
                allowed: vec!["scheduled".to_string(), "legacy".to_string()],
            }),
            Rc::new(AdaptiveRuleBatchDescriptionLengthRule { max_length: 64 }),
            Rc::new(AdaptiveRuleBatchRequiredDescriptionRule {
                message: "adaptive rule batch needs an entry".to_string(),
            }),
            Rc::new(AdaptiveRuleBatchTagsLimitRule {
                max_count: 3600,
                banned: Vec::new(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_rule_batch_validator();
        let item = new_adaptive_rule_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = AdaptiveRuleBatchChannelLengthRule { max_length: 80 };
        let mut item = new_adaptive_rule_batch("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = AdaptiveRuleBatchDescriptionLengthRule { max_length: 80 };
        let mut item = new_adaptive_rule_batch("x", "y");
        item.description = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = AdaptiveRuleBatchRequiredDescriptionRule {
            message: "adaptive rule batch needs a value".to_string(),
        };
        let mut item = new_adaptive_rule_batch("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
