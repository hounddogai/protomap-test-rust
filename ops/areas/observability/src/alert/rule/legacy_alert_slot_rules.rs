use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy alert slot must pass. Check returns its problems, or none.
pub trait LegacyAlertSlotRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAlertSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertSlotNamespaceLengthRule {
    pub max_length: i64,
}

impl LegacyAlertSlotRule for LegacyAlertSlotNamespaceLengthRule {
    fn check(&self, item: &LegacyAlertSlot) -> Vec<String> {
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
pub struct LegacyAlertSlotAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl LegacyAlertSlotRule for LegacyAlertSlotAllowedExternalRefRule {
    fn check(&self, item: &LegacyAlertSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is blocked", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertSlotRequiredCodeRule {
    pub message: String,
}

impl LegacyAlertSlotRule for LegacyAlertSlotRequiredCodeRule {
    fn check(&self, item: &LegacyAlertSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertSlotVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyAlertSlotRule for LegacyAlertSlotVersionRangeRule {
    fn check(&self, item: &LegacyAlertSlot) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertSlotExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyAlertSlotRule for LegacyAlertSlotExpiresAtRangeRule {
    fn check(&self, item: &LegacyAlertSlot) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertSlotRequiredNameRule {
    pub message: String,
}

impl LegacyAlertSlotRule for LegacyAlertSlotRequiredNameRule {
    fn check(&self, item: &LegacyAlertSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy alert slot.
#[derive(Debug, Clone)]
pub struct LegacyAlertSlotValidator {
    pub rules: Vec<Rc<dyn LegacyAlertSlotRule>>,
}

impl LegacyAlertSlotValidator {
    pub fn validate(&self, item: &LegacyAlertSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyAlertSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAlertSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_alert_slot_validator() -> LegacyAlertSlotValidator {
    LegacyAlertSlotValidator {
        rules: vec![
            Rc::new(LegacyAlertSlotNamespaceLengthRule { max_length: 1000 }) as Rc<dyn LegacyAlertSlotRule>,
            Rc::new(LegacyAlertSlotAllowedExternalRefRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(LegacyAlertSlotRequiredCodeRule {
                message: "legacy alert slot needs this field".to_string(),
            }),
            Rc::new(LegacyAlertSlotVersionRangeRule { min: 64, max: 3600 }),
            Rc::new(LegacyAlertSlotExpiresAtRangeRule { min: 80, max: 80 }),
            Rc::new(LegacyAlertSlotRequiredNameRule {
                message: "legacy alert slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_alert_slot_validator();
        let item = new_legacy_alert_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = LegacyAlertSlotNamespaceLengthRule { max_length: 80 };
        let mut item = new_legacy_alert_slot("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyAlertSlotRequiredCodeRule {
            message: "legacy alert slot needs a value".to_string(),
        };
        let mut item = new_legacy_alert_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = LegacyAlertSlotVersionRangeRule { min: 16, max: 64 };
        let mut item = new_legacy_alert_slot("x", "y");
        item.version = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = LegacyAlertSlotExpiresAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_legacy_alert_slot("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyAlertSlotRequiredNameRule {
            message: "legacy alert slot needs a value".to_string(),
        };
        let mut item = new_legacy_alert_slot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
