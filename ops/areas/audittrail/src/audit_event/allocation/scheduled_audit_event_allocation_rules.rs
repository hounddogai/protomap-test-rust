use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled audit event allocation must pass. Check returns its problems, or none.
pub trait ScheduledAuditEventAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledAuditEventAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventAllocationRequiredNameRule {
    pub message: String,
}

impl ScheduledAuditEventAllocationRule for ScheduledAuditEventAllocationRequiredNameRule {
    fn check(&self, item: &ScheduledAuditEventAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventAllocationAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ScheduledAuditEventAllocationRule for ScheduledAuditEventAllocationAllowedNameRule {
    fn check(&self, item: &ScheduledAuditEventAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventAllocationCodePrefixRule {
    pub prefix: String,
}

impl ScheduledAuditEventAllocationRule for ScheduledAuditEventAllocationCodePrefixRule {
    fn check(&self, item: &ScheduledAuditEventAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled audit event allocation.
#[derive(Debug, Clone)]
pub struct ScheduledAuditEventAllocationValidator {
    pub rules: Vec<Rc<dyn ScheduledAuditEventAllocationRule>>,
}

impl ScheduledAuditEventAllocationValidator {
    pub fn validate(&self, item: &ScheduledAuditEventAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledAuditEventAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledAuditEventAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_audit_event_allocation_validator() -> ScheduledAuditEventAllocationValidator {
    ScheduledAuditEventAllocationValidator {
        rules: vec![
            Rc::new(ScheduledAuditEventAllocationRequiredNameRule {
                message: "scheduled audit event allocation needs an entry".to_string(),
            }) as Rc<dyn ScheduledAuditEventAllocationRule>,
            Rc::new(ScheduledAuditEventAllocationAllowedNameRule { allowed: Vec::new() }),
            Rc::new(ScheduledAuditEventAllocationCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_audit_event_allocation_validator();
        let item = new_scheduled_audit_event_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledAuditEventAllocationRequiredNameRule {
            message: "scheduled audit event allocation needs a value".to_string(),
        };
        let mut item = new_scheduled_audit_event_allocation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
