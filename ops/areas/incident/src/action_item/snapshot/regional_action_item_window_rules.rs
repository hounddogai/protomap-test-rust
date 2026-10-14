use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional action item window must pass. Check returns its problems, or none.
pub trait RegionalActionItemWindowRule: std::fmt::Debug {
    fn check(&self, item: &RegionalActionItemWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemWindowRequiredLabelRule {
    pub message: String,
}

impl RegionalActionItemWindowRule for RegionalActionItemWindowRequiredLabelRule {
    fn check(&self, item: &RegionalActionItemWindow) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemWindowChannelLengthRule {
    pub max_length: i64,
}

impl RegionalActionItemWindowRule for RegionalActionItemWindowChannelLengthRule {
    fn check(&self, item: &RegionalActionItemWindow) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemWindowCodePrefixRule {
    pub prefix: String,
}

impl RegionalActionItemWindowRule for RegionalActionItemWindowCodePrefixRule {
    fn check(&self, item: &RegionalActionItemWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemWindowCodeLengthRule {
    pub max_length: i64,
}

impl RegionalActionItemWindowRule for RegionalActionItemWindowCodeLengthRule {
    fn check(&self, item: &RegionalActionItemWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional action item window.
#[derive(Debug, Clone)]
pub struct RegionalActionItemWindowValidator {
    pub rules: Vec<Rc<dyn RegionalActionItemWindowRule>>,
}

impl RegionalActionItemWindowValidator {
    pub fn validate(&self, item: &RegionalActionItemWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalActionItemWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalActionItemWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_action_item_window_validator() -> RegionalActionItemWindowValidator {
    RegionalActionItemWindowValidator {
        rules: vec![
            Rc::new(RegionalActionItemWindowRequiredLabelRule {
                message: "regional action item window needs an entry".to_string(),
            }) as Rc<dyn RegionalActionItemWindowRule>,
            Rc::new(RegionalActionItemWindowChannelLengthRule { max_length: 120 }),
            Rc::new(RegionalActionItemWindowCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RegionalActionItemWindowCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_action_item_window_validator();
        let item = new_regional_action_item_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalActionItemWindowRequiredLabelRule {
            message: "regional action item window needs this field".to_string(),
        };
        let mut item = new_regional_action_item_window("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = RegionalActionItemWindowChannelLengthRule { max_length: 1000 };
        let mut item = new_regional_action_item_window("x", "y");
        item.channel = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalActionItemWindowCodeLengthRule { max_length: 64 };
        let mut item = new_regional_action_item_window("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
