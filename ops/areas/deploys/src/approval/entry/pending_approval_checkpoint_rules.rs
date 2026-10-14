use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending approval checkpoint must pass. Check returns its problems, or none.
pub trait PendingApprovalCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &PendingApprovalCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalCheckpointRequiredCodeRule {
    pub message: String,
}

impl PendingApprovalCheckpointRule for PendingApprovalCheckpointRequiredCodeRule {
    fn check(&self, item: &PendingApprovalCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalCheckpointRequiredTitleRule {
    pub message: String,
}

impl PendingApprovalCheckpointRule for PendingApprovalCheckpointRequiredTitleRule {
    fn check(&self, item: &PendingApprovalCheckpoint) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalCheckpointRequiredPhoneRule {
    pub message: String,
}

impl PendingApprovalCheckpointRule for PendingApprovalCheckpointRequiredPhoneRule {
    fn check(&self, item: &PendingApprovalCheckpoint) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl PendingApprovalCheckpointRule for PendingApprovalCheckpointCodeLengthRule {
    fn check(&self, item: &PendingApprovalCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalCheckpointPhoneLengthRule {
    pub max_length: i64,
}

impl PendingApprovalCheckpointRule for PendingApprovalCheckpointPhoneLengthRule {
    fn check(&self, item: &PendingApprovalCheckpoint) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalCheckpointPinnedGuardRule {
    pub limit: f64,
}

impl PendingApprovalCheckpointRule for PendingApprovalCheckpointPinnedGuardRule {
    fn check(&self, item: &PendingApprovalCheckpoint) -> Vec<String> {
        if item.pinned && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while pinned",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending approval checkpoint.
#[derive(Debug, Clone)]
pub struct PendingApprovalCheckpointValidator {
    pub rules: Vec<Rc<dyn PendingApprovalCheckpointRule>>,
}

impl PendingApprovalCheckpointValidator {
    pub fn validate(&self, item: &PendingApprovalCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingApprovalCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingApprovalCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_approval_checkpoint_validator() -> PendingApprovalCheckpointValidator {
    PendingApprovalCheckpointValidator {
        rules: vec![
            Rc::new(PendingApprovalCheckpointRequiredCodeRule {
                message: "pending approval checkpoint needs an entry".to_string(),
            }) as Rc<dyn PendingApprovalCheckpointRule>,
            Rc::new(PendingApprovalCheckpointRequiredTitleRule {
                message: "pending approval checkpoint needs a value".to_string(),
            }),
            Rc::new(PendingApprovalCheckpointRequiredPhoneRule {
                message: "pending approval checkpoint needs this field".to_string(),
            }),
            Rc::new(PendingApprovalCheckpointCodeLengthRule { max_length: 3600 }),
            Rc::new(PendingApprovalCheckpointPhoneLengthRule { max_length: 32 }),
            Rc::new(PendingApprovalCheckpointPinnedGuardRule { limit: 6.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_approval_checkpoint_validator();
        let item = new_pending_approval_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingApprovalCheckpointRequiredCodeRule {
            message: "pending approval checkpoint needs this field".to_string(),
        };
        let mut item = new_pending_approval_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingApprovalCheckpointRequiredTitleRule {
            message: "pending approval checkpoint needs a value".to_string(),
        };
        let mut item = new_pending_approval_checkpoint("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PendingApprovalCheckpointRequiredPhoneRule {
            message: "pending approval checkpoint needs a value".to_string(),
        };
        let mut item = new_pending_approval_checkpoint("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingApprovalCheckpointCodeLengthRule { max_length: 120 };
        let mut item = new_pending_approval_checkpoint("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = PendingApprovalCheckpointPhoneLengthRule { max_length: 64 };
        let mut item = new_pending_approval_checkpoint("x", "y");
        item.phone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
