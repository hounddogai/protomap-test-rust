use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered alert slot must pass. Check returns its problems, or none.
pub trait TieredAlertSlotRule: std::fmt::Debug {
    fn check(&self, item: &TieredAlertSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertSlotCodeLengthRule {
    pub max_length: i64,
}

impl TieredAlertSlotRule for TieredAlertSlotCodeLengthRule {
    fn check(&self, item: &TieredAlertSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertSlotCodePrefixRule {
    pub prefix: String,
}

impl TieredAlertSlotRule for TieredAlertSlotCodePrefixRule {
    fn check(&self, item: &TieredAlertSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertSlotRequiredCodeRule {
    pub message: String,
}

impl TieredAlertSlotRule for TieredAlertSlotRequiredCodeRule {
    fn check(&self, item: &TieredAlertSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered alert slot.
#[derive(Debug, Clone)]
pub struct TieredAlertSlotValidator {
    pub rules: Vec<Rc<dyn TieredAlertSlotRule>>,
}

impl TieredAlertSlotValidator {
    pub fn validate(&self, item: &TieredAlertSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredAlertSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAlertSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_alert_slot_validator() -> TieredAlertSlotValidator {
    TieredAlertSlotValidator {
        rules: vec![
            Rc::new(TieredAlertSlotCodeLengthRule { max_length: 80 }) as Rc<dyn TieredAlertSlotRule>,
            Rc::new(TieredAlertSlotCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TieredAlertSlotRequiredCodeRule {
                message: "tiered alert slot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_alert_slot_validator();
        let item = new_tiered_alert_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredAlertSlotCodeLengthRule { max_length: 64 };
        let mut item = new_tiered_alert_slot("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredAlertSlotRequiredCodeRule {
            message: "tiered alert slot needs this field".to_string(),
        };
        let mut item = new_tiered_alert_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
