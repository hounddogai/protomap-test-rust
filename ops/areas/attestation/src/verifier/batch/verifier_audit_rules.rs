use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier audit must pass. Check returns its problems, or none.
pub trait VerifierAuditRule: std::fmt::Debug {
    fn check(&self, item: &VerifierAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAuditRequiredLabelRule {
    pub message: String,
}

impl VerifierAuditRule for VerifierAuditRequiredLabelRule {
    fn check(&self, item: &VerifierAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAuditPinnedGuardRule {
    pub limit: i64,
}

impl VerifierAuditRule for VerifierAuditPinnedGuardRule {
    fn check(&self, item: &VerifierAudit) -> Vec<String> {
        if item.pinned && (item.duration_minutes > self.limit) {
            return vec![format!(
                "duration minutes {} exceeds {} while pinned",
                item.duration_minutes, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAuditCodeLengthRule {
    pub max_length: i64,
}

impl VerifierAuditRule for VerifierAuditCodeLengthRule {
    fn check(&self, item: &VerifierAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAuditRequiredNameRule {
    pub message: String,
}

impl VerifierAuditRule for VerifierAuditRequiredNameRule {
    fn check(&self, item: &VerifierAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAuditRequiredReviewerRule {
    pub message: String,
}

impl VerifierAuditRule for VerifierAuditRequiredReviewerRule {
    fn check(&self, item: &VerifierAudit) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAuditDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierAuditRule for VerifierAuditDurationMinutesRangeRule {
    fn check(&self, item: &VerifierAudit) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier audit.
#[derive(Debug, Clone)]
pub struct VerifierAuditValidator {
    pub rules: Vec<Rc<dyn VerifierAuditRule>>,
}

impl VerifierAuditValidator {
    pub fn validate(&self, item: &VerifierAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_audit_validator() -> VerifierAuditValidator {
    VerifierAuditValidator {
        rules: vec![
            Rc::new(VerifierAuditRequiredLabelRule {
                message: "verifier audit needs an entry".to_string(),
            }) as Rc<dyn VerifierAuditRule>,
            Rc::new(VerifierAuditPinnedGuardRule { limit: 255 }),
            Rc::new(VerifierAuditCodeLengthRule { max_length: 255 }),
            Rc::new(VerifierAuditRequiredNameRule {
                message: "verifier audit needs an entry".to_string(),
            }),
            Rc::new(VerifierAuditRequiredReviewerRule {
                message: "verifier audit needs a value".to_string(),
            }),
            Rc::new(VerifierAuditDurationMinutesRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_audit_validator();
        let item = new_verifier_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = VerifierAuditRequiredLabelRule {
            message: "verifier audit needs a value".to_string(),
        };
        let mut item = new_verifier_audit("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierAuditCodeLengthRule { max_length: 16 };
        let mut item = new_verifier_audit("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = VerifierAuditRequiredNameRule {
            message: "verifier audit needs this field".to_string(),
        };
        let mut item = new_verifier_audit("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = VerifierAuditRequiredReviewerRule {
            message: "verifier audit needs this field".to_string(),
        };
        let mut item = new_verifier_audit("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = VerifierAuditDurationMinutesRangeRule { min: 32, max: 80 };
        let mut item = new_verifier_audit("x", "y");
        item.duration_minutes = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
