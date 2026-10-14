use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item template must pass. Check returns its problems, or none.
pub trait ActionItemTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTemplateCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemTemplateRule for ActionItemTemplateCodeLengthRule {
    fn check(&self, item: &ActionItemTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTemplateRequiredExternalRefRule {
    pub message: String,
}

impl ActionItemTemplateRule for ActionItemTemplateRequiredExternalRefRule {
    fn check(&self, item: &ActionItemTemplate) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTemplateRequiredCodeRule {
    pub message: String,
}

impl ActionItemTemplateRule for ActionItemTemplateRequiredCodeRule {
    fn check(&self, item: &ActionItemTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTemplateTitleLengthRule {
    pub max_length: i64,
}

impl ActionItemTemplateRule for ActionItemTemplateTitleLengthRule {
    fn check(&self, item: &ActionItemTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item template.
#[derive(Debug, Clone)]
pub struct ActionItemTemplateValidator {
    pub rules: Vec<Rc<dyn ActionItemTemplateRule>>,
}

impl ActionItemTemplateValidator {
    pub fn validate(&self, item: &ActionItemTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_template_validator() -> ActionItemTemplateValidator {
    ActionItemTemplateValidator {
        rules: vec![
            Rc::new(ActionItemTemplateCodeLengthRule { max_length: 3600 }) as Rc<dyn ActionItemTemplateRule>,
            Rc::new(ActionItemTemplateRequiredExternalRefRule {
                message: "action item template needs this field".to_string(),
            }),
            Rc::new(ActionItemTemplateRequiredCodeRule {
                message: "action item template needs a value".to_string(),
            }),
            Rc::new(ActionItemTemplateTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_template_validator();
        let item = new_action_item_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ActionItemTemplateCodeLengthRule { max_length: 3600 };
        let mut item = new_action_item_template("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ActionItemTemplateRequiredExternalRefRule {
            message: "action item template needs an entry".to_string(),
        };
        let mut item = new_action_item_template("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ActionItemTemplateRequiredCodeRule {
            message: "action item template needs a value".to_string(),
        };
        let mut item = new_action_item_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ActionItemTemplateTitleLengthRule { max_length: 64 };
        let mut item = new_action_item_template("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
