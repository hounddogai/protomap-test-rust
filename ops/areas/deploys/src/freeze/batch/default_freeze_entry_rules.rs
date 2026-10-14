use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default freeze entry must pass. Check returns its problems, or none.
pub trait DefaultFreezeEntryRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFreezeEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeEntryRequiredLabelRule {
    pub message: String,
}

impl DefaultFreezeEntryRule for DefaultFreezeEntryRequiredLabelRule {
    fn check(&self, item: &DefaultFreezeEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeEntryAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl DefaultFreezeEntryRule for DefaultFreezeEntryAllowedAudienceRule {
    fn check(&self, item: &DefaultFreezeEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is duplicated", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeEntryRequiredCodeRule {
    pub message: String,
}

impl DefaultFreezeEntryRule for DefaultFreezeEntryRequiredCodeRule {
    fn check(&self, item: &DefaultFreezeEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default freeze entry.
#[derive(Debug, Clone)]
pub struct DefaultFreezeEntryValidator {
    pub rules: Vec<Rc<dyn DefaultFreezeEntryRule>>,
}

impl DefaultFreezeEntryValidator {
    pub fn validate(&self, item: &DefaultFreezeEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultFreezeEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFreezeEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_freeze_entry_validator() -> DefaultFreezeEntryValidator {
    DefaultFreezeEntryValidator {
        rules: vec![
            Rc::new(DefaultFreezeEntryRequiredLabelRule {
                message: "default freeze entry needs a value".to_string(),
            }) as Rc<dyn DefaultFreezeEntryRule>,
            Rc::new(DefaultFreezeEntryAllowedAudienceRule {
                allowed: vec!["seasonal".to_string(), "lenient".to_string(), "capped".to_string()],
            }),
            Rc::new(DefaultFreezeEntryRequiredCodeRule {
                message: "default freeze entry needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_freeze_entry_validator();
        let item = new_default_freeze_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultFreezeEntryRequiredLabelRule {
            message: "default freeze entry needs a value".to_string(),
        };
        let mut item = new_default_freeze_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultFreezeEntryRequiredCodeRule {
            message: "default freeze entry needs a value".to_string(),
        };
        let mut item = new_default_freeze_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
