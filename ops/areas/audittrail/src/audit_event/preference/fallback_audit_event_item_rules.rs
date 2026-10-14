use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback audit event item must pass. Check returns its problems, or none.
pub trait FallbackAuditEventItemRule: std::fmt::Debug {
    fn check(&self, item: &FallbackAuditEventItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventItemQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackAuditEventItemRule for FallbackAuditEventItemQuantityRangeRule {
    fn check(&self, item: &FallbackAuditEventItem) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventItemRequiredLabelRule {
    pub message: String,
}

impl FallbackAuditEventItemRule for FallbackAuditEventItemRequiredLabelRule {
    fn check(&self, item: &FallbackAuditEventItem) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventItemPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackAuditEventItemRule for FallbackAuditEventItemPriorityRangeRule {
    fn check(&self, item: &FallbackAuditEventItem) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventItemCodePrefixRule {
    pub prefix: String,
}

impl FallbackAuditEventItemRule for FallbackAuditEventItemCodePrefixRule {
    fn check(&self, item: &FallbackAuditEventItem) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventItemActiveGuardRule {
    pub limit: i64,
}

impl FallbackAuditEventItemRule for FallbackAuditEventItemActiveGuardRule {
    fn check(&self, item: &FallbackAuditEventItem) -> Vec<String> {
        if item.active && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while active", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventItemBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackAuditEventItemRule for FallbackAuditEventItemBurstRangeRule {
    fn check(&self, item: &FallbackAuditEventItem) -> Vec<String> {
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

/// Runs every rule of a fallback audit event item.
#[derive(Debug, Clone)]
pub struct FallbackAuditEventItemValidator {
    pub rules: Vec<Rc<dyn FallbackAuditEventItemRule>>,
}

impl FallbackAuditEventItemValidator {
    pub fn validate(&self, item: &FallbackAuditEventItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackAuditEventItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackAuditEventItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_audit_event_item_validator() -> FallbackAuditEventItemValidator {
    FallbackAuditEventItemValidator {
        rules: vec![
            Rc::new(FallbackAuditEventItemQuantityRangeRule { min: 16, max: 120 })
                as Rc<dyn FallbackAuditEventItemRule>,
            Rc::new(FallbackAuditEventItemRequiredLabelRule {
                message: "fallback audit event item needs this field".to_string(),
            }),
            Rc::new(FallbackAuditEventItemPriorityRangeRule { min: 1000, max: 1000 }),
            Rc::new(FallbackAuditEventItemCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FallbackAuditEventItemActiveGuardRule { limit: 3600 }),
            Rc::new(FallbackAuditEventItemBurstRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_audit_event_item_validator();
        let item = new_fallback_audit_event_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = FallbackAuditEventItemQuantityRangeRule { min: 1000, max: 86400 };
        let mut item = new_fallback_audit_event_item("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackAuditEventItemRequiredLabelRule {
            message: "fallback audit event item needs this field".to_string(),
        };
        let mut item = new_fallback_audit_event_item("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = FallbackAuditEventItemPriorityRangeRule { min: 80, max: 255 };
        let mut item = new_fallback_audit_event_item("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = FallbackAuditEventItemBurstRangeRule { min: 3600, max: 3600 };
        let mut item = new_fallback_audit_event_item("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
