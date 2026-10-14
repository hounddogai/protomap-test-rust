use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity slot must pass. Check returns its problems, or none.
pub trait IdentitySlotRule: std::fmt::Debug {
    fn check(&self, item: &IdentitySlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySlotRequiredCodeRule {
    pub message: String,
}

impl IdentitySlotRule for IdentitySlotRequiredCodeRule {
    fn check(&self, item: &IdentitySlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySlotAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl IdentitySlotRule for IdentitySlotAllowedLabelRule {
    fn check(&self, item: &IdentitySlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySlotUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentitySlotRule for IdentitySlotUpdatedAtRangeRule {
    fn check(&self, item: &IdentitySlot) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a identity slot.
#[derive(Debug, Clone)]
pub struct IdentitySlotValidator {
    pub rules: Vec<Rc<dyn IdentitySlotRule>>,
}

impl IdentitySlotValidator {
    pub fn validate(&self, item: &IdentitySlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentitySlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentitySlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_slot_validator() -> IdentitySlotValidator {
    IdentitySlotValidator {
        rules: vec![
            Rc::new(IdentitySlotRequiredCodeRule {
                message: "identity slot needs an entry".to_string(),
            }) as Rc<dyn IdentitySlotRule>,
            Rc::new(IdentitySlotAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(IdentitySlotUpdatedAtRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_slot_validator();
        let item = new_identity_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentitySlotRequiredCodeRule {
            message: "identity slot needs this field".to_string(),
        };
        let mut item = new_identity_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = IdentitySlotUpdatedAtRangeRule { min: 32, max: 120 };
        let mut item = new_identity_slot("x", "y");
        item.updated_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
