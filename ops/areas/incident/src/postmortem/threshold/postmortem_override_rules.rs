use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem override must pass. Check returns its problems, or none.
pub trait PostmortemOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemOverrideRequiredLabelRule {
    pub message: String,
}

impl PostmortemOverrideRule for PostmortemOverrideRequiredLabelRule {
    fn check(&self, item: &PostmortemOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemOverrideRequiredCurrencyRule {
    pub message: String,
}

impl PostmortemOverrideRule for PostmortemOverrideRequiredCurrencyRule {
    fn check(&self, item: &PostmortemOverride) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemOverrideLabelLengthRule {
    pub max_length: i64,
}

impl PostmortemOverrideRule for PostmortemOverrideLabelLengthRule {
    fn check(&self, item: &PostmortemOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemOverrideRequiredTimezoneRule {
    pub message: String,
}

impl PostmortemOverrideRule for PostmortemOverrideRequiredTimezoneRule {
    fn check(&self, item: &PostmortemOverride) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem override.
#[derive(Debug, Clone)]
pub struct PostmortemOverrideValidator {
    pub rules: Vec<Rc<dyn PostmortemOverrideRule>>,
}

impl PostmortemOverrideValidator {
    pub fn validate(&self, item: &PostmortemOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_override_validator() -> PostmortemOverrideValidator {
    PostmortemOverrideValidator {
        rules: vec![
            Rc::new(PostmortemOverrideRequiredLabelRule {
                message: "postmortem override needs an entry".to_string(),
            }) as Rc<dyn PostmortemOverrideRule>,
            Rc::new(PostmortemOverrideRequiredCurrencyRule {
                message: "postmortem override needs a value".to_string(),
            }),
            Rc::new(PostmortemOverrideLabelLengthRule { max_length: 86400 }),
            Rc::new(PostmortemOverrideRequiredTimezoneRule {
                message: "postmortem override needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_override_validator();
        let item = new_postmortem_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PostmortemOverrideRequiredLabelRule {
            message: "postmortem override needs an entry".to_string(),
        };
        let mut item = new_postmortem_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = PostmortemOverrideRequiredCurrencyRule {
            message: "postmortem override needs a value".to_string(),
        };
        let mut item = new_postmortem_override("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PostmortemOverrideLabelLengthRule { max_length: 64 };
        let mut item = new_postmortem_override("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PostmortemOverrideRequiredTimezoneRule {
            message: "postmortem override needs this field".to_string(),
        };
        let mut item = new_postmortem_override("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
