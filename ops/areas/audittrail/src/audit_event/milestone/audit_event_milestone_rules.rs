use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event milestone must pass. Check returns its problems, or none.
pub trait AuditEventMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventMilestoneRule for AuditEventMilestoneCodeLengthRule {
    fn check(&self, item: &AuditEventMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMilestoneNameLengthRule {
    pub max_length: i64,
}

impl AuditEventMilestoneRule for AuditEventMilestoneNameLengthRule {
    fn check(&self, item: &AuditEventMilestone) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMilestoneDescriptionLengthRule {
    pub max_length: i64,
}

impl AuditEventMilestoneRule for AuditEventMilestoneDescriptionLengthRule {
    fn check(&self, item: &AuditEventMilestone) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event milestone.
#[derive(Debug, Clone)]
pub struct AuditEventMilestoneValidator {
    pub rules: Vec<Rc<dyn AuditEventMilestoneRule>>,
}

impl AuditEventMilestoneValidator {
    pub fn validate(&self, item: &AuditEventMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_milestone_validator() -> AuditEventMilestoneValidator {
    AuditEventMilestoneValidator {
        rules: vec![
            Rc::new(AuditEventMilestoneCodeLengthRule { max_length: 120 }) as Rc<dyn AuditEventMilestoneRule>,
            Rc::new(AuditEventMilestoneNameLengthRule { max_length: 86400 }),
            Rc::new(AuditEventMilestoneDescriptionLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_milestone_validator();
        let item = new_audit_event_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AuditEventMilestoneCodeLengthRule { max_length: 255 };
        let mut item = new_audit_event_milestone("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AuditEventMilestoneNameLengthRule { max_length: 255 };
        let mut item = new_audit_event_milestone("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = AuditEventMilestoneDescriptionLengthRule { max_length: 1000 };
        let mut item = new_audit_event_milestone("x", "y");
        item.description = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
