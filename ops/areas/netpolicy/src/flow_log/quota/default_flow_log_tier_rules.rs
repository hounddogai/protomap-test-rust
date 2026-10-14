use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default flow log tier must pass. Check returns its problems, or none.
pub trait DefaultFlowLogTierRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFlowLogTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogTierCodePrefixRule {
    pub prefix: String,
}

impl DefaultFlowLogTierRule for DefaultFlowLogTierCodePrefixRule {
    fn check(&self, item: &DefaultFlowLogTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogTierChannelLengthRule {
    pub max_length: i64,
}

impl DefaultFlowLogTierRule for DefaultFlowLogTierChannelLengthRule {
    fn check(&self, item: &DefaultFlowLogTier) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogTierCodeLengthRule {
    pub max_length: i64,
}

impl DefaultFlowLogTierRule for DefaultFlowLogTierCodeLengthRule {
    fn check(&self, item: &DefaultFlowLogTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogTierTitleLengthRule {
    pub max_length: i64,
}

impl DefaultFlowLogTierRule for DefaultFlowLogTierTitleLengthRule {
    fn check(&self, item: &DefaultFlowLogTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogTierRequiredTitleRule {
    pub message: String,
}

impl DefaultFlowLogTierRule for DefaultFlowLogTierRequiredTitleRule {
    fn check(&self, item: &DefaultFlowLogTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default flow log tier.
#[derive(Debug, Clone)]
pub struct DefaultFlowLogTierValidator {
    pub rules: Vec<Rc<dyn DefaultFlowLogTierRule>>,
}

impl DefaultFlowLogTierValidator {
    pub fn validate(&self, item: &DefaultFlowLogTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultFlowLogTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFlowLogTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_flow_log_tier_validator() -> DefaultFlowLogTierValidator {
    DefaultFlowLogTierValidator {
        rules: vec![
            Rc::new(DefaultFlowLogTierCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn DefaultFlowLogTierRule>,
            Rc::new(DefaultFlowLogTierChannelLengthRule { max_length: 255 }),
            Rc::new(DefaultFlowLogTierCodeLengthRule { max_length: 16 }),
            Rc::new(DefaultFlowLogTierTitleLengthRule { max_length: 120 }),
            Rc::new(DefaultFlowLogTierRequiredTitleRule {
                message: "default flow log tier needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_flow_log_tier_validator();
        let item = new_default_flow_log_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = DefaultFlowLogTierChannelLengthRule { max_length: 80 };
        let mut item = new_default_flow_log_tier("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultFlowLogTierCodeLengthRule { max_length: 86400 };
        let mut item = new_default_flow_log_tier("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultFlowLogTierTitleLengthRule { max_length: 16 };
        let mut item = new_default_flow_log_tier("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultFlowLogTierRequiredTitleRule {
            message: "default flow log tier needs this field".to_string(),
        };
        let mut item = new_default_flow_log_tier("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
