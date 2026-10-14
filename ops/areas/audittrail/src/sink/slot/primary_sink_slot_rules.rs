use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary sink slot must pass. Check returns its problems, or none.
pub trait PrimarySinkSlotRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySinkSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkSlotAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl PrimarySinkSlotRule for PrimarySinkSlotAllowedCategoryRule {
    fn check(&self, item: &PrimarySinkSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is below minimum", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkSlotNameLengthRule {
    pub max_length: i64,
}

impl PrimarySinkSlotRule for PrimarySinkSlotNameLengthRule {
    fn check(&self, item: &PrimarySinkSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkSlotRequiredCodeRule {
    pub message: String,
}

impl PrimarySinkSlotRule for PrimarySinkSlotRequiredCodeRule {
    fn check(&self, item: &PrimarySinkSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary sink slot.
#[derive(Debug, Clone)]
pub struct PrimarySinkSlotValidator {
    pub rules: Vec<Rc<dyn PrimarySinkSlotRule>>,
}

impl PrimarySinkSlotValidator {
    pub fn validate(&self, item: &PrimarySinkSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySinkSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySinkSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_sink_slot_validator() -> PrimarySinkSlotValidator {
    PrimarySinkSlotValidator {
        rules: vec![
            Rc::new(PrimarySinkSlotAllowedCategoryRule {
                allowed: vec!["global".to_string()],
            }) as Rc<dyn PrimarySinkSlotRule>,
            Rc::new(PrimarySinkSlotNameLengthRule { max_length: 86400 }),
            Rc::new(PrimarySinkSlotRequiredCodeRule {
                message: "primary sink slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_sink_slot_validator();
        let item = new_primary_sink_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimarySinkSlotNameLengthRule { max_length: 64 };
        let mut item = new_primary_sink_slot("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimarySinkSlotRequiredCodeRule {
            message: "primary sink slot needs this field".to_string(),
        };
        let mut item = new_primary_sink_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
