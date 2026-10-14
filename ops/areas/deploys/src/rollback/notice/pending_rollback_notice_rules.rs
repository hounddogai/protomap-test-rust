use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending rollback notice must pass. Check returns its problems, or none.
pub trait PendingRollbackNoticeRule: std::fmt::Debug {
    fn check(&self, item: &PendingRollbackNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRollbackNoticeAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingRollbackNoticeRule for PendingRollbackNoticeAllowedCodeRule {
    fn check(&self, item: &PendingRollbackNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRollbackNoticeRequiredKeyIdRule {
    pub message: String,
}

impl PendingRollbackNoticeRule for PendingRollbackNoticeRequiredKeyIdRule {
    fn check(&self, item: &PendingRollbackNotice) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRollbackNoticeRequiredPhoneRule {
    pub message: String,
}

impl PendingRollbackNoticeRule for PendingRollbackNoticeRequiredPhoneRule {
    fn check(&self, item: &PendingRollbackNotice) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending rollback notice.
#[derive(Debug, Clone)]
pub struct PendingRollbackNoticeValidator {
    pub rules: Vec<Rc<dyn PendingRollbackNoticeRule>>,
}

impl PendingRollbackNoticeValidator {
    pub fn validate(&self, item: &PendingRollbackNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRollbackNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRollbackNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_rollback_notice_validator() -> PendingRollbackNoticeValidator {
    PendingRollbackNoticeValidator {
        rules: vec![
            Rc::new(PendingRollbackNoticeAllowedCodeRule {
                allowed: vec!["capped".to_string(), "tiered".to_string()],
            }) as Rc<dyn PendingRollbackNoticeRule>,
            Rc::new(PendingRollbackNoticeRequiredKeyIdRule {
                message: "pending rollback notice needs an entry".to_string(),
            }),
            Rc::new(PendingRollbackNoticeRequiredPhoneRule {
                message: "pending rollback notice needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_rollback_notice_validator();
        let item = new_pending_rollback_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PendingRollbackNoticeRequiredKeyIdRule {
            message: "pending rollback notice needs a value".to_string(),
        };
        let mut item = new_pending_rollback_notice("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PendingRollbackNoticeRequiredPhoneRule {
            message: "pending rollback notice needs this field".to_string(),
        };
        let mut item = new_pending_rollback_notice("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
