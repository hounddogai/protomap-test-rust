use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary binding slot must pass. Check returns its problems, or none.
pub trait PrimaryBindingSlotRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryBindingSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSlotRegionLengthRule {
    pub max_length: i64,
}

impl PrimaryBindingSlotRule for PrimaryBindingSlotRegionLengthRule {
    fn check(&self, item: &PrimaryBindingSlot) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSlotCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryBindingSlotRule for PrimaryBindingSlotCodeLengthRule {
    fn check(&self, item: &PrimaryBindingSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSlotRequiredCodeRule {
    pub message: String,
}

impl PrimaryBindingSlotRule for PrimaryBindingSlotRequiredCodeRule {
    fn check(&self, item: &PrimaryBindingSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSlotCodePrefixRule {
    pub prefix: String,
}

impl PrimaryBindingSlotRule for PrimaryBindingSlotCodePrefixRule {
    fn check(&self, item: &PrimaryBindingSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSlotSummaryLengthRule {
    pub max_length: i64,
}

impl PrimaryBindingSlotRule for PrimaryBindingSlotSummaryLengthRule {
    fn check(&self, item: &PrimaryBindingSlot) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSlotRequiredRegionRule {
    pub message: String,
}

impl PrimaryBindingSlotRule for PrimaryBindingSlotRequiredRegionRule {
    fn check(&self, item: &PrimaryBindingSlot) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary binding slot.
#[derive(Debug, Clone)]
pub struct PrimaryBindingSlotValidator {
    pub rules: Vec<Rc<dyn PrimaryBindingSlotRule>>,
}

impl PrimaryBindingSlotValidator {
    pub fn validate(&self, item: &PrimaryBindingSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryBindingSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryBindingSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_binding_slot_validator() -> PrimaryBindingSlotValidator {
    PrimaryBindingSlotValidator {
        rules: vec![
            Rc::new(PrimaryBindingSlotRegionLengthRule { max_length: 255 }) as Rc<dyn PrimaryBindingSlotRule>,
            Rc::new(PrimaryBindingSlotCodeLengthRule { max_length: 16 }),
            Rc::new(PrimaryBindingSlotRequiredCodeRule {
                message: "primary binding slot needs this field".to_string(),
            }),
            Rc::new(PrimaryBindingSlotCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PrimaryBindingSlotSummaryLengthRule { max_length: 32 }),
            Rc::new(PrimaryBindingSlotRequiredRegionRule {
                message: "primary binding slot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_binding_slot_validator();
        let item = new_primary_binding_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PrimaryBindingSlotRegionLengthRule { max_length: 3600 };
        let mut item = new_primary_binding_slot("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryBindingSlotCodeLengthRule { max_length: 16 };
        let mut item = new_primary_binding_slot("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryBindingSlotRequiredCodeRule {
            message: "primary binding slot needs this field".to_string(),
        };
        let mut item = new_primary_binding_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = PrimaryBindingSlotSummaryLengthRule { max_length: 1000 };
        let mut item = new_primary_binding_slot("x", "y");
        item.summary = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = PrimaryBindingSlotRequiredRegionRule {
            message: "primary binding slot needs this field".to_string(),
        };
        let mut item = new_primary_binding_slot("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
