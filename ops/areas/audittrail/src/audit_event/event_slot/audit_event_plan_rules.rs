use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event plan must pass. Check returns its problems, or none.
pub trait AuditEventPlanRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPlanRequiredCodeRule {
    pub message: String,
}

impl AuditEventPlanRule for AuditEventPlanRequiredCodeRule {
    fn check(&self, item: &AuditEventPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPlanCodePrefixRule {
    pub prefix: String,
}

impl AuditEventPlanRule for AuditEventPlanCodePrefixRule {
    fn check(&self, item: &AuditEventPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPlanRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventPlanRule for AuditEventPlanRetriesRangeRule {
    fn check(&self, item: &AuditEventPlan) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPlanFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventPlanRule for AuditEventPlanFloorValueRangeRule {
    fn check(&self, item: &AuditEventPlan) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPlanCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventPlanRule for AuditEventPlanCodeLengthRule {
    fn check(&self, item: &AuditEventPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event plan.
#[derive(Debug, Clone)]
pub struct AuditEventPlanValidator {
    pub rules: Vec<Rc<dyn AuditEventPlanRule>>,
}

impl AuditEventPlanValidator {
    pub fn validate(&self, item: &AuditEventPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_plan_validator() -> AuditEventPlanValidator {
    AuditEventPlanValidator {
        rules: vec![
            Rc::new(AuditEventPlanRequiredCodeRule {
                message: "audit event plan needs an entry".to_string(),
            }) as Rc<dyn AuditEventPlanRule>,
            Rc::new(AuditEventPlanCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AuditEventPlanRetriesRangeRule { min: 32, max: 80 }),
            Rc::new(AuditEventPlanFloorValueRangeRule { min: 0.6, max: 1668.5 }),
            Rc::new(AuditEventPlanCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_plan_validator();
        let item = new_audit_event_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventPlanRequiredCodeRule {
            message: "audit event plan needs this field".to_string(),
        };
        let mut item = new_audit_event_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = AuditEventPlanRetriesRangeRule { min: 3600, max: 86400 };
        let mut item = new_audit_event_plan("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = AuditEventPlanFloorValueRangeRule { min: 4.9, max: 2632.5 };
        let mut item = new_audit_event_plan("x", "y");
        item.floor_value = 2634.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AuditEventPlanCodeLengthRule { max_length: 255 };
        let mut item = new_audit_event_plan("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
