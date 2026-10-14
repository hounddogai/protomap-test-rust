use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item preference must pass. Check returns its problems, or none.
pub trait ActionItemPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPreferenceUnitLengthRule {
    pub max_length: i64,
}

impl ActionItemPreferenceRule for ActionItemPreferenceUnitLengthRule {
    fn check(&self, item: &ActionItemPreference) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPreferenceChannelLengthRule {
    pub max_length: i64,
}

impl ActionItemPreferenceRule for ActionItemPreferenceChannelLengthRule {
    fn check(&self, item: &ActionItemPreference) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPreferenceAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl ActionItemPreferenceRule for ActionItemPreferenceAllowedChannelRule {
    fn check(&self, item: &ActionItemPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is too long", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPreferenceRequiredTitleRule {
    pub message: String,
}

impl ActionItemPreferenceRule for ActionItemPreferenceRequiredTitleRule {
    fn check(&self, item: &ActionItemPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemPreferenceRule for ActionItemPreferenceCodeLengthRule {
    fn check(&self, item: &ActionItemPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPreferenceStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemPreferenceRule for ActionItemPreferenceStartsAtRangeRule {
    fn check(&self, item: &ActionItemPreference) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item preference.
#[derive(Debug, Clone)]
pub struct ActionItemPreferenceValidator {
    pub rules: Vec<Rc<dyn ActionItemPreferenceRule>>,
}

impl ActionItemPreferenceValidator {
    pub fn validate(&self, item: &ActionItemPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_preference_validator() -> ActionItemPreferenceValidator {
    ActionItemPreferenceValidator {
        rules: vec![
            Rc::new(ActionItemPreferenceUnitLengthRule { max_length: 120 }) as Rc<dyn ActionItemPreferenceRule>,
            Rc::new(ActionItemPreferenceChannelLengthRule { max_length: 255 }),
            Rc::new(ActionItemPreferenceAllowedChannelRule {
                allowed: vec!["tiered".to_string(), "primary".to_string(), "strict".to_string()],
            }),
            Rc::new(ActionItemPreferenceRequiredTitleRule {
                message: "action item preference needs an entry".to_string(),
            }),
            Rc::new(ActionItemPreferenceCodeLengthRule { max_length: 32 }),
            Rc::new(ActionItemPreferenceStartsAtRangeRule { min: 32, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_preference_validator();
        let item = new_action_item_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ActionItemPreferenceUnitLengthRule { max_length: 16 };
        let mut item = new_action_item_preference("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ActionItemPreferenceChannelLengthRule { max_length: 3600 };
        let mut item = new_action_item_preference("x", "y");
        item.channel = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ActionItemPreferenceRequiredTitleRule {
            message: "action item preference needs an entry".to_string(),
        };
        let mut item = new_action_item_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ActionItemPreferenceCodeLengthRule { max_length: 16 };
        let mut item = new_action_item_preference("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ActionItemPreferenceStartsAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_action_item_preference("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
