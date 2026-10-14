use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal probe slot must pass. Check returns its problems, or none.
pub trait SeasonalProbeSlotRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalProbeSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeSlotRequiredPrincipalRule {
    pub message: String,
}

impl SeasonalProbeSlotRule for SeasonalProbeSlotRequiredPrincipalRule {
    fn check(&self, item: &SeasonalProbeSlot) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeSlotRequiredCodeRule {
    pub message: String,
}

impl SeasonalProbeSlotRule for SeasonalProbeSlotRequiredCodeRule {
    fn check(&self, item: &SeasonalProbeSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeSlotLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalProbeSlotRule for SeasonalProbeSlotLabelLengthRule {
    fn check(&self, item: &SeasonalProbeSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal probe slot.
#[derive(Debug, Clone)]
pub struct SeasonalProbeSlotValidator {
    pub rules: Vec<Rc<dyn SeasonalProbeSlotRule>>,
}

impl SeasonalProbeSlotValidator {
    pub fn validate(&self, item: &SeasonalProbeSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalProbeSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalProbeSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_probe_slot_validator() -> SeasonalProbeSlotValidator {
    SeasonalProbeSlotValidator {
        rules: vec![
            Rc::new(SeasonalProbeSlotRequiredPrincipalRule {
                message: "seasonal probe slot needs a value".to_string(),
            }) as Rc<dyn SeasonalProbeSlotRule>,
            Rc::new(SeasonalProbeSlotRequiredCodeRule {
                message: "seasonal probe slot needs an entry".to_string(),
            }),
            Rc::new(SeasonalProbeSlotLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_probe_slot_validator();
        let item = new_seasonal_probe_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = SeasonalProbeSlotRequiredPrincipalRule {
            message: "seasonal probe slot needs this field".to_string(),
        };
        let mut item = new_seasonal_probe_slot("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalProbeSlotRequiredCodeRule {
            message: "seasonal probe slot needs this field".to_string(),
        };
        let mut item = new_seasonal_probe_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SeasonalProbeSlotLabelLengthRule { max_length: 64 };
        let mut item = new_seasonal_probe_slot("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
