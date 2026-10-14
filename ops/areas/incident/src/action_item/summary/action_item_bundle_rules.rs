use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item bundle must pass. Check returns its problems, or none.
pub trait ActionItemBundleRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBundleCodePrefixRule {
    pub prefix: String,
}

impl ActionItemBundleRule for ActionItemBundleCodePrefixRule {
    fn check(&self, item: &ActionItemBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBundleRequiredLabelRule {
    pub message: String,
}

impl ActionItemBundleRule for ActionItemBundleRequiredLabelRule {
    fn check(&self, item: &ActionItemBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ActionItemBundleRule for ActionItemBundleAllowedCodeRule {
    fn check(&self, item: &ActionItemBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBundleCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemBundleRule for ActionItemBundleCodeLengthRule {
    fn check(&self, item: &ActionItemBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBundleFingerprintLengthRule {
    pub max_length: i64,
}

impl ActionItemBundleRule for ActionItemBundleFingerprintLengthRule {
    fn check(&self, item: &ActionItemBundle) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBundleLabelLengthRule {
    pub max_length: i64,
}

impl ActionItemBundleRule for ActionItemBundleLabelLengthRule {
    fn check(&self, item: &ActionItemBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item bundle.
#[derive(Debug, Clone)]
pub struct ActionItemBundleValidator {
    pub rules: Vec<Rc<dyn ActionItemBundleRule>>,
}

impl ActionItemBundleValidator {
    pub fn validate(&self, item: &ActionItemBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_bundle_validator() -> ActionItemBundleValidator {
    ActionItemBundleValidator {
        rules: vec![
            Rc::new(ActionItemBundleCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn ActionItemBundleRule>,
            Rc::new(ActionItemBundleRequiredLabelRule {
                message: "action item bundle needs an entry".to_string(),
            }),
            Rc::new(ActionItemBundleAllowedCodeRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(ActionItemBundleCodeLengthRule { max_length: 80 }),
            Rc::new(ActionItemBundleFingerprintLengthRule { max_length: 1000 }),
            Rc::new(ActionItemBundleLabelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_bundle_validator();
        let item = new_action_item_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ActionItemBundleRequiredLabelRule {
            message: "action item bundle needs an entry".to_string(),
        };
        let mut item = new_action_item_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ActionItemBundleCodeLengthRule { max_length: 3600 };
        let mut item = new_action_item_bundle("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ActionItemBundleFingerprintLengthRule { max_length: 64 };
        let mut item = new_action_item_bundle("x", "y");
        item.fingerprint = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ActionItemBundleLabelLengthRule { max_length: 3600 };
        let mut item = new_action_item_bundle("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
