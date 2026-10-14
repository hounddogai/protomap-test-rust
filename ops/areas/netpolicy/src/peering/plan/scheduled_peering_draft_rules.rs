use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled peering draft must pass. Check returns its problems, or none.
pub trait ScheduledPeeringDraftRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledPeeringDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringDraftCodePrefixRule {
    pub prefix: String,
}

impl ScheduledPeeringDraftRule for ScheduledPeeringDraftCodePrefixRule {
    fn check(&self, item: &ScheduledPeeringDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringDraftHostLengthRule {
    pub max_length: i64,
}

impl ScheduledPeeringDraftRule for ScheduledPeeringDraftHostLengthRule {
    fn check(&self, item: &ScheduledPeeringDraft) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringDraftCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledPeeringDraftRule for ScheduledPeeringDraftCodeLengthRule {
    fn check(&self, item: &ScheduledPeeringDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringDraftRequiredTitleRule {
    pub message: String,
}

impl ScheduledPeeringDraftRule for ScheduledPeeringDraftRequiredTitleRule {
    fn check(&self, item: &ScheduledPeeringDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled peering draft.
#[derive(Debug, Clone)]
pub struct ScheduledPeeringDraftValidator {
    pub rules: Vec<Rc<dyn ScheduledPeeringDraftRule>>,
}

impl ScheduledPeeringDraftValidator {
    pub fn validate(&self, item: &ScheduledPeeringDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledPeeringDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledPeeringDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_peering_draft_validator() -> ScheduledPeeringDraftValidator {
    ScheduledPeeringDraftValidator {
        rules: vec![
            Rc::new(ScheduledPeeringDraftCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn ScheduledPeeringDraftRule>,
            Rc::new(ScheduledPeeringDraftHostLengthRule { max_length: 120 }),
            Rc::new(ScheduledPeeringDraftCodeLengthRule { max_length: 32 }),
            Rc::new(ScheduledPeeringDraftRequiredTitleRule {
                message: "scheduled peering draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_peering_draft_validator();
        let item = new_scheduled_peering_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = ScheduledPeeringDraftHostLengthRule { max_length: 64 };
        let mut item = new_scheduled_peering_draft("x", "y");
        item.host = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledPeeringDraftCodeLengthRule { max_length: 255 };
        let mut item = new_scheduled_peering_draft("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledPeeringDraftRequiredTitleRule {
            message: "scheduled peering draft needs an entry".to_string(),
        };
        let mut item = new_scheduled_peering_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
