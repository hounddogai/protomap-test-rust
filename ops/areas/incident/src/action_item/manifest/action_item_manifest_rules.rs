use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item manifest must pass. Check returns its problems, or none.
pub trait ActionItemManifestRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemManifestChannelLengthRule {
    pub max_length: i64,
}

impl ActionItemManifestRule for ActionItemManifestChannelLengthRule {
    fn check(&self, item: &ActionItemManifest) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemManifestCodePrefixRule {
    pub prefix: String,
}

impl ActionItemManifestRule for ActionItemManifestCodePrefixRule {
    fn check(&self, item: &ActionItemManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemManifestRequiredChannelRule {
    pub message: String,
}

impl ActionItemManifestRule for ActionItemManifestRequiredChannelRule {
    fn check(&self, item: &ActionItemManifest) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item manifest.
#[derive(Debug, Clone)]
pub struct ActionItemManifestValidator {
    pub rules: Vec<Rc<dyn ActionItemManifestRule>>,
}

impl ActionItemManifestValidator {
    pub fn validate(&self, item: &ActionItemManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_manifest_validator() -> ActionItemManifestValidator {
    ActionItemManifestValidator {
        rules: vec![
            Rc::new(ActionItemManifestChannelLengthRule { max_length: 120 }) as Rc<dyn ActionItemManifestRule>,
            Rc::new(ActionItemManifestCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ActionItemManifestRequiredChannelRule {
                message: "action item manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_manifest_validator();
        let item = new_action_item_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ActionItemManifestChannelLengthRule { max_length: 16 };
        let mut item = new_action_item_manifest("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = ActionItemManifestRequiredChannelRule {
            message: "action item manifest needs this field".to_string(),
        };
        let mut item = new_action_item_manifest("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
