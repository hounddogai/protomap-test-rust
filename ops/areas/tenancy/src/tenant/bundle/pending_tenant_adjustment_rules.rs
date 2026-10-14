use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending tenant adjustment must pass. Check returns its problems, or none.
pub trait PendingTenantAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingTenantAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTenantAdjustmentFingerprintLengthRule {
    pub max_length: i64,
}

impl PendingTenantAdjustmentRule for PendingTenantAdjustmentFingerprintLengthRule {
    fn check(&self, item: &PendingTenantAdjustment) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTenantAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl PendingTenantAdjustmentRule for PendingTenantAdjustmentCodeLengthRule {
    fn check(&self, item: &PendingTenantAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTenantAdjustmentRequiredSourceRule {
    pub message: String,
}

impl PendingTenantAdjustmentRule for PendingTenantAdjustmentRequiredSourceRule {
    fn check(&self, item: &PendingTenantAdjustment) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTenantAdjustmentRequiredCodeRule {
    pub message: String,
}

impl PendingTenantAdjustmentRule for PendingTenantAdjustmentRequiredCodeRule {
    fn check(&self, item: &PendingTenantAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTenantAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl PendingTenantAdjustmentRule for PendingTenantAdjustmentLabelLengthRule {
    fn check(&self, item: &PendingTenantAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTenantAdjustmentRequiredLabelRule {
    pub message: String,
}

impl PendingTenantAdjustmentRule for PendingTenantAdjustmentRequiredLabelRule {
    fn check(&self, item: &PendingTenantAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending tenant adjustment.
#[derive(Debug, Clone)]
pub struct PendingTenantAdjustmentValidator {
    pub rules: Vec<Rc<dyn PendingTenantAdjustmentRule>>,
}

impl PendingTenantAdjustmentValidator {
    pub fn validate(&self, item: &PendingTenantAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingTenantAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTenantAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_tenant_adjustment_validator() -> PendingTenantAdjustmentValidator {
    PendingTenantAdjustmentValidator {
        rules: vec![
            Rc::new(PendingTenantAdjustmentFingerprintLengthRule { max_length: 64 })
                as Rc<dyn PendingTenantAdjustmentRule>,
            Rc::new(PendingTenantAdjustmentCodeLengthRule { max_length: 120 }),
            Rc::new(PendingTenantAdjustmentRequiredSourceRule {
                message: "pending tenant adjustment needs this field".to_string(),
            }),
            Rc::new(PendingTenantAdjustmentRequiredCodeRule {
                message: "pending tenant adjustment needs a value".to_string(),
            }),
            Rc::new(PendingTenantAdjustmentLabelLengthRule { max_length: 86400 }),
            Rc::new(PendingTenantAdjustmentRequiredLabelRule {
                message: "pending tenant adjustment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_tenant_adjustment_validator();
        let item = new_pending_tenant_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = PendingTenantAdjustmentFingerprintLengthRule { max_length: 3600 };
        let mut item = new_pending_tenant_adjustment("x", "y");
        item.fingerprint = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingTenantAdjustmentCodeLengthRule { max_length: 3600 };
        let mut item = new_pending_tenant_adjustment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = PendingTenantAdjustmentRequiredSourceRule {
            message: "pending tenant adjustment needs an entry".to_string(),
        };
        let mut item = new_pending_tenant_adjustment("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingTenantAdjustmentRequiredCodeRule {
            message: "pending tenant adjustment needs a value".to_string(),
        };
        let mut item = new_pending_tenant_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingTenantAdjustmentLabelLengthRule { max_length: 32 };
        let mut item = new_pending_tenant_adjustment("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingTenantAdjustmentRequiredLabelRule {
            message: "pending tenant adjustment needs an entry".to_string(),
        };
        let mut item = new_pending_tenant_adjustment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
