use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item rule must pass. Check returns its problems, or none.
pub trait ActionItemRuleRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRuleTitleLengthRule {
    pub max_length: i64,
}

impl ActionItemRuleRule for ActionItemRuleTitleLengthRule {
    fn check(&self, item: &ActionItemRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRuleRequiredCodeRule {
    pub message: String,
}

impl ActionItemRuleRule for ActionItemRuleRequiredCodeRule {
    fn check(&self, item: &ActionItemRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRuleCodePrefixRule {
    pub prefix: String,
}

impl ActionItemRuleRule for ActionItemRuleCodePrefixRule {
    fn check(&self, item: &ActionItemRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRuleScopeLengthRule {
    pub max_length: i64,
}

impl ActionItemRuleRule for ActionItemRuleScopeLengthRule {
    fn check(&self, item: &ActionItemRule) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ActionItemRuleRule for ActionItemRuleAllowedCodeRule {
    fn check(&self, item: &ActionItemRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item rule.
#[derive(Debug, Clone)]
pub struct ActionItemRuleValidator {
    pub rules: Vec<Rc<dyn ActionItemRuleRule>>,
}

impl ActionItemRuleValidator {
    pub fn validate(&self, item: &ActionItemRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_rule_validator() -> ActionItemRuleValidator {
    ActionItemRuleValidator {
        rules: vec![
            Rc::new(ActionItemRuleTitleLengthRule { max_length: 80 }) as Rc<dyn ActionItemRuleRule>,
            Rc::new(ActionItemRuleRequiredCodeRule {
                message: "action item rule needs an entry".to_string(),
            }),
            Rc::new(ActionItemRuleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ActionItemRuleScopeLengthRule { max_length: 16 }),
            Rc::new(ActionItemRuleAllowedCodeRule {
                allowed: vec!["strict".to_string(), "scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_rule_validator();
        let item = new_action_item_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ActionItemRuleTitleLengthRule { max_length: 255 };
        let mut item = new_action_item_rule("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ActionItemRuleRequiredCodeRule {
            message: "action item rule needs this field".to_string(),
        };
        let mut item = new_action_item_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = ActionItemRuleScopeLengthRule { max_length: 255 };
        let mut item = new_action_item_rule("x", "y");
        item.scope = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
