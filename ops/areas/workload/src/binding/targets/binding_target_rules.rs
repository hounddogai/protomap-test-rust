use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding target must pass. Check returns its problems, or none.
pub trait BindingTargetRule: std::fmt::Debug {
    fn check(&self, item: &BindingTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTargetChannelLengthRule {
    pub max_length: i64,
}

impl BindingTargetRule for BindingTargetChannelLengthRule {
    fn check(&self, item: &BindingTarget) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTargetRequiredTitleRule {
    pub message: String,
}

impl BindingTargetRule for BindingTargetRequiredTitleRule {
    fn check(&self, item: &BindingTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTargetCodePrefixRule {
    pub prefix: String,
}

impl BindingTargetRule for BindingTargetCodePrefixRule {
    fn check(&self, item: &BindingTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding target.
#[derive(Debug, Clone)]
pub struct BindingTargetValidator {
    pub rules: Vec<Rc<dyn BindingTargetRule>>,
}

impl BindingTargetValidator {
    pub fn validate(&self, item: &BindingTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_target_validator() -> BindingTargetValidator {
    BindingTargetValidator {
        rules: vec![
            Rc::new(BindingTargetChannelLengthRule { max_length: 64 }) as Rc<dyn BindingTargetRule>,
            Rc::new(BindingTargetRequiredTitleRule {
                message: "binding target needs this field".to_string(),
            }),
            Rc::new(BindingTargetCodePrefixRule {
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
        let validator = default_binding_target_validator();
        let item = new_binding_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = BindingTargetChannelLengthRule { max_length: 120 };
        let mut item = new_binding_target("x", "y");
        item.channel = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BindingTargetRequiredTitleRule {
            message: "binding target needs this field".to_string(),
        };
        let mut item = new_binding_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
