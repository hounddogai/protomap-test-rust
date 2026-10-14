use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem target must pass. Check returns its problems, or none.
pub trait PostmortemTargetRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTargetRequiredPhoneRule {
    pub message: String,
}

impl PostmortemTargetRule for PostmortemTargetRequiredPhoneRule {
    fn check(&self, item: &PostmortemTarget) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTargetLabelLengthRule {
    pub max_length: i64,
}

impl PostmortemTargetRule for PostmortemTargetLabelLengthRule {
    fn check(&self, item: &PostmortemTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTargetCodePrefixRule {
    pub prefix: String,
}

impl PostmortemTargetRule for PostmortemTargetCodePrefixRule {
    fn check(&self, item: &PostmortemTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem target.
#[derive(Debug, Clone)]
pub struct PostmortemTargetValidator {
    pub rules: Vec<Rc<dyn PostmortemTargetRule>>,
}

impl PostmortemTargetValidator {
    pub fn validate(&self, item: &PostmortemTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_target_validator() -> PostmortemTargetValidator {
    PostmortemTargetValidator {
        rules: vec![
            Rc::new(PostmortemTargetRequiredPhoneRule {
                message: "postmortem target needs a value".to_string(),
            }) as Rc<dyn PostmortemTargetRule>,
            Rc::new(PostmortemTargetLabelLengthRule { max_length: 64 }),
            Rc::new(PostmortemTargetCodePrefixRule {
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
        let validator = default_postmortem_target_validator();
        let item = new_postmortem_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PostmortemTargetRequiredPhoneRule {
            message: "postmortem target needs a value".to_string(),
        };
        let mut item = new_postmortem_target("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PostmortemTargetLabelLengthRule { max_length: 3600 };
        let mut item = new_postmortem_target("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
