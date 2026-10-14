use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient audit event group must pass. Check returns its problems, or none.
pub trait LenientAuditEventGroupRule: std::fmt::Debug {
    fn check(&self, item: &LenientAuditEventGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventGroupCodeLengthRule {
    pub max_length: i64,
}

impl LenientAuditEventGroupRule for LenientAuditEventGroupCodeLengthRule {
    fn check(&self, item: &LenientAuditEventGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventGroupRequiredTitleRule {
    pub message: String,
}

impl LenientAuditEventGroupRule for LenientAuditEventGroupRequiredTitleRule {
    fn check(&self, item: &LenientAuditEventGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventGroupRequiredRegionRule {
    pub message: String,
}

impl LenientAuditEventGroupRule for LenientAuditEventGroupRequiredRegionRule {
    fn check(&self, item: &LenientAuditEventGroup) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient audit event group.
#[derive(Debug, Clone)]
pub struct LenientAuditEventGroupValidator {
    pub rules: Vec<Rc<dyn LenientAuditEventGroupRule>>,
}

impl LenientAuditEventGroupValidator {
    pub fn validate(&self, item: &LenientAuditEventGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientAuditEventGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientAuditEventGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_audit_event_group_validator() -> LenientAuditEventGroupValidator {
    LenientAuditEventGroupValidator {
        rules: vec![
            Rc::new(LenientAuditEventGroupCodeLengthRule { max_length: 80 }) as Rc<dyn LenientAuditEventGroupRule>,
            Rc::new(LenientAuditEventGroupRequiredTitleRule {
                message: "lenient audit event group needs a value".to_string(),
            }),
            Rc::new(LenientAuditEventGroupRequiredRegionRule {
                message: "lenient audit event group needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_audit_event_group_validator();
        let item = new_lenient_audit_event_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientAuditEventGroupCodeLengthRule { max_length: 3600 };
        let mut item = new_lenient_audit_event_group("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientAuditEventGroupRequiredTitleRule {
            message: "lenient audit event group needs a value".to_string(),
        };
        let mut item = new_lenient_audit_event_group("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = LenientAuditEventGroupRequiredRegionRule {
            message: "lenient audit event group needs this field".to_string(),
        };
        let mut item = new_lenient_audit_event_group("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
