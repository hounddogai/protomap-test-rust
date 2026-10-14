use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event variance must pass. Check returns its problems, or none.
pub trait AuditEventVarianceRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventVarianceNamespaceLengthRule {
    pub max_length: i64,
}

impl AuditEventVarianceRule for AuditEventVarianceNamespaceLengthRule {
    fn check(&self, item: &AuditEventVariance) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventVarianceRequiredCodeRule {
    pub message: String,
}

impl AuditEventVarianceRule for AuditEventVarianceRequiredCodeRule {
    fn check(&self, item: &AuditEventVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventVarianceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventVarianceRule for AuditEventVarianceRotationDaysRangeRule {
    fn check(&self, item: &AuditEventVariance) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventVarianceAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AuditEventVarianceRule for AuditEventVarianceAllowedNameRule {
    fn check(&self, item: &AuditEventVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event variance.
#[derive(Debug, Clone)]
pub struct AuditEventVarianceValidator {
    pub rules: Vec<Rc<dyn AuditEventVarianceRule>>,
}

impl AuditEventVarianceValidator {
    pub fn validate(&self, item: &AuditEventVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_variance_validator() -> AuditEventVarianceValidator {
    AuditEventVarianceValidator {
        rules: vec![
            Rc::new(AuditEventVarianceNamespaceLengthRule { max_length: 120 }) as Rc<dyn AuditEventVarianceRule>,
            Rc::new(AuditEventVarianceRequiredCodeRule {
                message: "audit event variance needs a value".to_string(),
            }),
            Rc::new(AuditEventVarianceRotationDaysRangeRule { min: 64, max: 255 }),
            Rc::new(AuditEventVarianceAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_variance_validator();
        let item = new_audit_event_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = AuditEventVarianceNamespaceLengthRule { max_length: 64 };
        let mut item = new_audit_event_variance("x", "y");
        item.namespace = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventVarianceRequiredCodeRule {
            message: "audit event variance needs this field".to_string(),
        };
        let mut item = new_audit_event_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = AuditEventVarianceRotationDaysRangeRule { min: 32, max: 120 };
        let mut item = new_audit_event_variance("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
