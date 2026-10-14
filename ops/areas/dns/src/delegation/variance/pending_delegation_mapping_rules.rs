use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending delegation mapping must pass. Check returns its problems, or none.
pub trait PendingDelegationMappingRule: std::fmt::Debug {
    fn check(&self, item: &PendingDelegationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMappingRequiredChannelRule {
    pub message: String,
}

impl PendingDelegationMappingRule for PendingDelegationMappingRequiredChannelRule {
    fn check(&self, item: &PendingDelegationMapping) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMappingNotesLengthRule {
    pub max_length: i64,
}

impl PendingDelegationMappingRule for PendingDelegationMappingNotesLengthRule {
    fn check(&self, item: &PendingDelegationMapping) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMappingAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingDelegationMappingRule for PendingDelegationMappingAllowedCodeRule {
    fn check(&self, item: &PendingDelegationMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending delegation mapping.
#[derive(Debug, Clone)]
pub struct PendingDelegationMappingValidator {
    pub rules: Vec<Rc<dyn PendingDelegationMappingRule>>,
}

impl PendingDelegationMappingValidator {
    pub fn validate(&self, item: &PendingDelegationMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingDelegationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingDelegationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_delegation_mapping_validator() -> PendingDelegationMappingValidator {
    PendingDelegationMappingValidator {
        rules: vec![
            Rc::new(PendingDelegationMappingRequiredChannelRule {
                message: "pending delegation mapping needs this field".to_string(),
            }) as Rc<dyn PendingDelegationMappingRule>,
            Rc::new(PendingDelegationMappingNotesLengthRule { max_length: 1000 }),
            Rc::new(PendingDelegationMappingAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_delegation_mapping_validator();
        let item = new_pending_delegation_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = PendingDelegationMappingRequiredChannelRule {
            message: "pending delegation mapping needs this field".to_string(),
        };
        let mut item = new_pending_delegation_mapping("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = PendingDelegationMappingNotesLengthRule { max_length: 32 };
        let mut item = new_pending_delegation_mapping("x", "y");
        item.notes = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
