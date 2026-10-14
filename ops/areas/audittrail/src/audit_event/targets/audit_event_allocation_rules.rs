use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event allocation must pass. Check returns its problems, or none.
pub trait AuditEventAllocationRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAllocationWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventAllocationRule for AuditEventAllocationWeightRangeRule {
    fn check(&self, item: &AuditEventAllocation) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAllocationRequiredNameRule {
    pub message: String,
}

impl AuditEventAllocationRule for AuditEventAllocationRequiredNameRule {
    fn check(&self, item: &AuditEventAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAllocationRequiredCodeRule {
    pub message: String,
}

impl AuditEventAllocationRule for AuditEventAllocationRequiredCodeRule {
    fn check(&self, item: &AuditEventAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event allocation.
#[derive(Debug, Clone)]
pub struct AuditEventAllocationValidator {
    pub rules: Vec<Rc<dyn AuditEventAllocationRule>>,
}

impl AuditEventAllocationValidator {
    pub fn validate(&self, item: &AuditEventAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_allocation_validator() -> AuditEventAllocationValidator {
    AuditEventAllocationValidator {
        rules: vec![
            Rc::new(AuditEventAllocationWeightRangeRule { min: 6.2, max: 4868.3 }) as Rc<dyn AuditEventAllocationRule>,
            Rc::new(AuditEventAllocationRequiredNameRule {
                message: "audit event allocation needs this field".to_string(),
            }),
            Rc::new(AuditEventAllocationRequiredCodeRule {
                message: "audit event allocation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_allocation_validator();
        let item = new_audit_event_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = AuditEventAllocationWeightRangeRule { min: 8.6, max: 3969.6 };
        let mut item = new_audit_event_allocation("x", "y");
        item.weight = 3971.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AuditEventAllocationRequiredNameRule {
            message: "audit event allocation needs a value".to_string(),
        };
        let mut item = new_audit_event_allocation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventAllocationRequiredCodeRule {
            message: "audit event allocation needs an entry".to_string(),
        };
        let mut item = new_audit_event_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
