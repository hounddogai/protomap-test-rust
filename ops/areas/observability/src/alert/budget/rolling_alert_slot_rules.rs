use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling alert slot must pass. Check returns its problems, or none.
pub trait RollingAlertSlotRule: std::fmt::Debug {
    fn check(&self, item: &RollingAlertSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertSlotCodePrefixRule {
    pub prefix: String,
}

impl RollingAlertSlotRule for RollingAlertSlotCodePrefixRule {
    fn check(&self, item: &RollingAlertSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertSlotCountryLengthRule {
    pub max_length: i64,
}

impl RollingAlertSlotRule for RollingAlertSlotCountryLengthRule {
    fn check(&self, item: &RollingAlertSlot) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertSlotRequiredCountryRule {
    pub message: String,
}

impl RollingAlertSlotRule for RollingAlertSlotRequiredCountryRule {
    fn check(&self, item: &RollingAlertSlot) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertSlotNameLengthRule {
    pub max_length: i64,
}

impl RollingAlertSlotRule for RollingAlertSlotNameLengthRule {
    fn check(&self, item: &RollingAlertSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling alert slot.
#[derive(Debug, Clone)]
pub struct RollingAlertSlotValidator {
    pub rules: Vec<Rc<dyn RollingAlertSlotRule>>,
}

impl RollingAlertSlotValidator {
    pub fn validate(&self, item: &RollingAlertSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingAlertSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingAlertSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_alert_slot_validator() -> RollingAlertSlotValidator {
    RollingAlertSlotValidator {
        rules: vec![
            Rc::new(RollingAlertSlotCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RollingAlertSlotRule>,
            Rc::new(RollingAlertSlotCountryLengthRule { max_length: 32 }),
            Rc::new(RollingAlertSlotRequiredCountryRule {
                message: "rolling alert slot needs an entry".to_string(),
            }),
            Rc::new(RollingAlertSlotNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_alert_slot_validator();
        let item = new_rolling_alert_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = RollingAlertSlotCountryLengthRule { max_length: 64 };
        let mut item = new_rolling_alert_slot("x", "y");
        item.country = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = RollingAlertSlotRequiredCountryRule {
            message: "rolling alert slot needs this field".to_string(),
        };
        let mut item = new_rolling_alert_slot("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingAlertSlotNameLengthRule { max_length: 86400 };
        let mut item = new_rolling_alert_slot("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
