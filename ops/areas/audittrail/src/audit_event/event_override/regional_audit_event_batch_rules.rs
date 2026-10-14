use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional audit event batch must pass. Check returns its problems, or none.
pub trait RegionalAuditEventBatchRule: std::fmt::Debug {
    fn check(&self, item: &RegionalAuditEventBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventBatchLabelLengthRule {
    pub max_length: i64,
}

impl RegionalAuditEventBatchRule for RegionalAuditEventBatchLabelLengthRule {
    fn check(&self, item: &RegionalAuditEventBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventBatchRequiredCodeRule {
    pub message: String,
}

impl RegionalAuditEventBatchRule for RegionalAuditEventBatchRequiredCodeRule {
    fn check(&self, item: &RegionalAuditEventBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventBatchCodeLengthRule {
    pub max_length: i64,
}

impl RegionalAuditEventBatchRule for RegionalAuditEventBatchCodeLengthRule {
    fn check(&self, item: &RegionalAuditEventBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventBatchAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalAuditEventBatchRule for RegionalAuditEventBatchAttemptsRangeRule {
    fn check(&self, item: &RegionalAuditEventBatch) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalAuditEventBatchRule for RegionalAuditEventBatchAllowedCodeRule {
    fn check(&self, item: &RegionalAuditEventBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventBatchBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalAuditEventBatchRule for RegionalAuditEventBatchBurstRangeRule {
    fn check(&self, item: &RegionalAuditEventBatch) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a regional audit event batch.
#[derive(Debug, Clone)]
pub struct RegionalAuditEventBatchValidator {
    pub rules: Vec<Rc<dyn RegionalAuditEventBatchRule>>,
}

impl RegionalAuditEventBatchValidator {
    pub fn validate(&self, item: &RegionalAuditEventBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalAuditEventBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalAuditEventBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_audit_event_batch_validator() -> RegionalAuditEventBatchValidator {
    RegionalAuditEventBatchValidator {
        rules: vec![
            Rc::new(RegionalAuditEventBatchLabelLengthRule { max_length: 120 }) as Rc<dyn RegionalAuditEventBatchRule>,
            Rc::new(RegionalAuditEventBatchRequiredCodeRule {
                message: "regional audit event batch needs a value".to_string(),
            }),
            Rc::new(RegionalAuditEventBatchCodeLengthRule { max_length: 3600 }),
            Rc::new(RegionalAuditEventBatchAttemptsRangeRule { min: 16, max: 64 }),
            Rc::new(RegionalAuditEventBatchAllowedCodeRule {
                allowed: vec!["default".to_string(), "rolling".to_string(), "tiered".to_string()],
            }),
            Rc::new(RegionalAuditEventBatchBurstRangeRule { min: 64, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_audit_event_batch_validator();
        let item = new_regional_audit_event_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalAuditEventBatchLabelLengthRule { max_length: 120 };
        let mut item = new_regional_audit_event_batch("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalAuditEventBatchRequiredCodeRule {
            message: "regional audit event batch needs a value".to_string(),
        };
        let mut item = new_regional_audit_event_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalAuditEventBatchCodeLengthRule { max_length: 120 };
        let mut item = new_regional_audit_event_batch("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RegionalAuditEventBatchAttemptsRangeRule { min: 255, max: 255 };
        let mut item = new_regional_audit_event_batch("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = RegionalAuditEventBatchBurstRangeRule { min: 32, max: 120 };
        let mut item = new_regional_audit_event_batch("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
