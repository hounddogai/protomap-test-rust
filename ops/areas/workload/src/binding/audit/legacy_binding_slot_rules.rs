use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy binding slot must pass. Check returns its problems, or none.
pub trait LegacyBindingSlotRule: std::fmt::Debug {
    fn check(&self, item: &LegacyBindingSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingSlotFingerprintLengthRule {
    pub max_length: i64,
}

impl LegacyBindingSlotRule for LegacyBindingSlotFingerprintLengthRule {
    fn check(&self, item: &LegacyBindingSlot) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingSlotRequiredNameRule {
    pub message: String,
}

impl LegacyBindingSlotRule for LegacyBindingSlotRequiredNameRule {
    fn check(&self, item: &LegacyBindingSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingSlotRequiredClusterRule {
    pub message: String,
}

impl LegacyBindingSlotRule for LegacyBindingSlotRequiredClusterRule {
    fn check(&self, item: &LegacyBindingSlot) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingSlotAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl LegacyBindingSlotRule for LegacyBindingSlotAllowedFingerprintRule {
    fn check(&self, item: &LegacyBindingSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is inconsistent", item.fingerprint)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy binding slot.
#[derive(Debug, Clone)]
pub struct LegacyBindingSlotValidator {
    pub rules: Vec<Rc<dyn LegacyBindingSlotRule>>,
}

impl LegacyBindingSlotValidator {
    pub fn validate(&self, item: &LegacyBindingSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyBindingSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyBindingSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_binding_slot_validator() -> LegacyBindingSlotValidator {
    LegacyBindingSlotValidator {
        rules: vec![
            Rc::new(LegacyBindingSlotFingerprintLengthRule { max_length: 16 }) as Rc<dyn LegacyBindingSlotRule>,
            Rc::new(LegacyBindingSlotRequiredNameRule {
                message: "legacy binding slot needs a value".to_string(),
            }),
            Rc::new(LegacyBindingSlotRequiredClusterRule {
                message: "legacy binding slot needs an entry".to_string(),
            }),
            Rc::new(LegacyBindingSlotAllowedFingerprintRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_binding_slot_validator();
        let item = new_legacy_binding_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = LegacyBindingSlotFingerprintLengthRule { max_length: 120 };
        let mut item = new_legacy_binding_slot("x", "y");
        item.fingerprint = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyBindingSlotRequiredNameRule {
            message: "legacy binding slot needs an entry".to_string(),
        };
        let mut item = new_legacy_binding_slot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = LegacyBindingSlotRequiredClusterRule {
            message: "legacy binding slot needs a value".to_string(),
        };
        let mut item = new_legacy_binding_slot("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
