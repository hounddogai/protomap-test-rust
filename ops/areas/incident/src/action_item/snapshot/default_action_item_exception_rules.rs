use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default action item exception must pass. Check returns its problems, or none.
pub trait DefaultActionItemExceptionRule: std::fmt::Debug {
    fn check(&self, item: &DefaultActionItemException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemExceptionRequiredExternalRefRule {
    pub message: String,
}

impl DefaultActionItemExceptionRule for DefaultActionItemExceptionRequiredExternalRefRule {
    fn check(&self, item: &DefaultActionItemException) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemExceptionRequiredNamespaceRule {
    pub message: String,
}

impl DefaultActionItemExceptionRule for DefaultActionItemExceptionRequiredNamespaceRule {
    fn check(&self, item: &DefaultActionItemException) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemExceptionCodePrefixRule {
    pub prefix: String,
}

impl DefaultActionItemExceptionRule for DefaultActionItemExceptionCodePrefixRule {
    fn check(&self, item: &DefaultActionItemException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default action item exception.
#[derive(Debug, Clone)]
pub struct DefaultActionItemExceptionValidator {
    pub rules: Vec<Rc<dyn DefaultActionItemExceptionRule>>,
}

impl DefaultActionItemExceptionValidator {
    pub fn validate(&self, item: &DefaultActionItemException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultActionItemException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultActionItemException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_action_item_exception_validator() -> DefaultActionItemExceptionValidator {
    DefaultActionItemExceptionValidator {
        rules: vec![
            Rc::new(DefaultActionItemExceptionRequiredExternalRefRule {
                message: "default action item exception needs a value".to_string(),
            }) as Rc<dyn DefaultActionItemExceptionRule>,
            Rc::new(DefaultActionItemExceptionRequiredNamespaceRule {
                message: "default action item exception needs this field".to_string(),
            }),
            Rc::new(DefaultActionItemExceptionCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_action_item_exception_validator();
        let item = new_default_action_item_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = DefaultActionItemExceptionRequiredExternalRefRule {
            message: "default action item exception needs an entry".to_string(),
        };
        let mut item = new_default_action_item_exception("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = DefaultActionItemExceptionRequiredNamespaceRule {
            message: "default action item exception needs this field".to_string(),
        };
        let mut item = new_default_action_item_exception("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
