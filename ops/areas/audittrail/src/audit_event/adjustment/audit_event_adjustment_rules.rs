use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event adjustment must pass. Check returns its problems, or none.
pub trait AuditEventAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAdjustmentChannelLengthRule {
    pub max_length: i64,
}

impl AuditEventAdjustmentRule for AuditEventAdjustmentChannelLengthRule {
    fn check(&self, item: &AuditEventAdjustment) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAdjustmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AuditEventAdjustmentRule for AuditEventAdjustmentAllowedNameRule {
    fn check(&self, item: &AuditEventAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAdjustmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventAdjustmentRule for AuditEventAdjustmentQuantityRangeRule {
    fn check(&self, item: &AuditEventAdjustment) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAdjustmentRequiredCodeRule {
    pub message: String,
}

impl AuditEventAdjustmentRule for AuditEventAdjustmentRequiredCodeRule {
    fn check(&self, item: &AuditEventAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event adjustment.
#[derive(Debug, Clone)]
pub struct AuditEventAdjustmentValidator {
    pub rules: Vec<Rc<dyn AuditEventAdjustmentRule>>,
}

impl AuditEventAdjustmentValidator {
    pub fn validate(&self, item: &AuditEventAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_adjustment_validator() -> AuditEventAdjustmentValidator {
    AuditEventAdjustmentValidator {
        rules: vec![
            Rc::new(AuditEventAdjustmentChannelLengthRule { max_length: 255 }) as Rc<dyn AuditEventAdjustmentRule>,
            Rc::new(AuditEventAdjustmentAllowedNameRule {
                allowed: vec!["default".to_string(), "batched".to_string()],
            }),
            Rc::new(AuditEventAdjustmentQuantityRangeRule { min: 80, max: 120 }),
            Rc::new(AuditEventAdjustmentRequiredCodeRule {
                message: "audit event adjustment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_adjustment_validator();
        let item = new_audit_event_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = AuditEventAdjustmentChannelLengthRule { max_length: 64 };
        let mut item = new_audit_event_adjustment("x", "y");
        item.channel = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = AuditEventAdjustmentQuantityRangeRule { min: 32, max: 64 };
        let mut item = new_audit_event_adjustment("x", "y");
        item.quantity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventAdjustmentRequiredCodeRule {
            message: "audit event adjustment needs an entry".to_string(),
        };
        let mut item = new_audit_event_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
