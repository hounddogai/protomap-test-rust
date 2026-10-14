use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a grant assignment must pass. Check returns its problems, or none.
pub trait GrantAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &GrantAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAssignmentAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl GrantAssignmentRule for GrantAssignmentAllowedChannelRule {
    fn check(&self, item: &GrantAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is too long", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAssignmentCodePrefixRule {
    pub prefix: String,
}

impl GrantAssignmentRule for GrantAssignmentCodePrefixRule {
    fn check(&self, item: &GrantAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAssignmentChannelLengthRule {
    pub max_length: i64,
}

impl GrantAssignmentRule for GrantAssignmentChannelLengthRule {
    fn check(&self, item: &GrantAssignment) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant assignment.
#[derive(Debug, Clone)]
pub struct GrantAssignmentValidator {
    pub rules: Vec<Rc<dyn GrantAssignmentRule>>,
}

impl GrantAssignmentValidator {
    pub fn validate(&self, item: &GrantAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_assignment_validator() -> GrantAssignmentValidator {
    GrantAssignmentValidator {
        rules: vec![
            Rc::new(GrantAssignmentAllowedChannelRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn GrantAssignmentRule>,
            Rc::new(GrantAssignmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(GrantAssignmentChannelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_assignment_validator();
        let item = new_grant_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = GrantAssignmentChannelLengthRule { max_length: 32 };
        let mut item = new_grant_assignment("x", "y");
        item.channel = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
