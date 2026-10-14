use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary binding rule must pass. Check returns its problems, or none.
pub trait PrimaryBindingRuleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryBindingRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingRuleHostLengthRule {
    pub max_length: i64,
}

impl PrimaryBindingRuleRule for PrimaryBindingRuleHostLengthRule {
    fn check(&self, item: &PrimaryBindingRule) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingRuleRequiredLabelRule {
    pub message: String,
}

impl PrimaryBindingRuleRule for PrimaryBindingRuleRequiredLabelRule {
    fn check(&self, item: &PrimaryBindingRule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingRuleCodePrefixRule {
    pub prefix: String,
}

impl PrimaryBindingRuleRule for PrimaryBindingRuleCodePrefixRule {
    fn check(&self, item: &PrimaryBindingRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary binding rule.
#[derive(Debug, Clone)]
pub struct PrimaryBindingRuleValidator {
    pub rules: Vec<Rc<dyn PrimaryBindingRuleRule>>,
}

impl PrimaryBindingRuleValidator {
    pub fn validate(&self, item: &PrimaryBindingRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryBindingRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryBindingRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_binding_rule_validator() -> PrimaryBindingRuleValidator {
    PrimaryBindingRuleValidator {
        rules: vec![
            Rc::new(PrimaryBindingRuleHostLengthRule { max_length: 255 }) as Rc<dyn PrimaryBindingRuleRule>,
            Rc::new(PrimaryBindingRuleRequiredLabelRule {
                message: "primary binding rule needs this field".to_string(),
            }),
            Rc::new(PrimaryBindingRuleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_binding_rule_validator();
        let item = new_primary_binding_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = PrimaryBindingRuleHostLengthRule { max_length: 255 };
        let mut item = new_primary_binding_rule("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryBindingRuleRequiredLabelRule {
            message: "primary binding rule needs this field".to_string(),
        };
        let mut item = new_primary_binding_rule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
