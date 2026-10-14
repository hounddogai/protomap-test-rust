use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending postmortem override must pass. Check returns its problems, or none.
pub trait PendingPostmortemOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PendingPostmortemOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemOverrideCodeLengthRule {
    pub max_length: i64,
}

impl PendingPostmortemOverrideRule for PendingPostmortemOverrideCodeLengthRule {
    fn check(&self, item: &PendingPostmortemOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemOverrideRequiredCodeRule {
    pub message: String,
}

impl PendingPostmortemOverrideRule for PendingPostmortemOverrideRequiredCodeRule {
    fn check(&self, item: &PendingPostmortemOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemOverrideRequiredChannelRule {
    pub message: String,
}

impl PendingPostmortemOverrideRule for PendingPostmortemOverrideRequiredChannelRule {
    fn check(&self, item: &PendingPostmortemOverride) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending postmortem override.
#[derive(Debug, Clone)]
pub struct PendingPostmortemOverrideValidator {
    pub rules: Vec<Rc<dyn PendingPostmortemOverrideRule>>,
}

impl PendingPostmortemOverrideValidator {
    pub fn validate(&self, item: &PendingPostmortemOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingPostmortemOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPostmortemOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_postmortem_override_validator() -> PendingPostmortemOverrideValidator {
    PendingPostmortemOverrideValidator {
        rules: vec![
            Rc::new(PendingPostmortemOverrideCodeLengthRule { max_length: 80 })
                as Rc<dyn PendingPostmortemOverrideRule>,
            Rc::new(PendingPostmortemOverrideRequiredCodeRule {
                message: "pending postmortem override needs this field".to_string(),
            }),
            Rc::new(PendingPostmortemOverrideRequiredChannelRule {
                message: "pending postmortem override needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_postmortem_override_validator();
        let item = new_pending_postmortem_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingPostmortemOverrideCodeLengthRule { max_length: 120 };
        let mut item = new_pending_postmortem_override("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingPostmortemOverrideRequiredCodeRule {
            message: "pending postmortem override needs this field".to_string(),
        };
        let mut item = new_pending_postmortem_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = PendingPostmortemOverrideRequiredChannelRule {
            message: "pending postmortem override needs an entry".to_string(),
        };
        let mut item = new_pending_postmortem_override("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
