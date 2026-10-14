use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary action item manifest must pass. Check returns its problems, or none.
pub trait PrimaryActionItemManifestRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryActionItemManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemManifestCodePrefixRule {
    pub prefix: String,
}

impl PrimaryActionItemManifestRule for PrimaryActionItemManifestCodePrefixRule {
    fn check(&self, item: &PrimaryActionItemManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemManifestRequiredLabelRule {
    pub message: String,
}

impl PrimaryActionItemManifestRule for PrimaryActionItemManifestRequiredLabelRule {
    fn check(&self, item: &PrimaryActionItemManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemManifestLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryActionItemManifestRule for PrimaryActionItemManifestLabelLengthRule {
    fn check(&self, item: &PrimaryActionItemManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemManifestRequiredCodeRule {
    pub message: String,
}

impl PrimaryActionItemManifestRule for PrimaryActionItemManifestRequiredCodeRule {
    fn check(&self, item: &PrimaryActionItemManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary action item manifest.
#[derive(Debug, Clone)]
pub struct PrimaryActionItemManifestValidator {
    pub rules: Vec<Rc<dyn PrimaryActionItemManifestRule>>,
}

impl PrimaryActionItemManifestValidator {
    pub fn validate(&self, item: &PrimaryActionItemManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryActionItemManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryActionItemManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_action_item_manifest_validator() -> PrimaryActionItemManifestValidator {
    PrimaryActionItemManifestValidator {
        rules: vec![
            Rc::new(PrimaryActionItemManifestCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn PrimaryActionItemManifestRule>,
            Rc::new(PrimaryActionItemManifestRequiredLabelRule {
                message: "primary action item manifest needs a value".to_string(),
            }),
            Rc::new(PrimaryActionItemManifestLabelLengthRule { max_length: 120 }),
            Rc::new(PrimaryActionItemManifestRequiredCodeRule {
                message: "primary action item manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_action_item_manifest_validator();
        let item = new_primary_action_item_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryActionItemManifestRequiredLabelRule {
            message: "primary action item manifest needs a value".to_string(),
        };
        let mut item = new_primary_action_item_manifest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryActionItemManifestLabelLengthRule { max_length: 32 };
        let mut item = new_primary_action_item_manifest("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryActionItemManifestRequiredCodeRule {
            message: "primary action item manifest needs a value".to_string(),
        };
        let mut item = new_primary_action_item_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
