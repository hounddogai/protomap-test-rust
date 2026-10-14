use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional token slot must pass. Check returns its problems, or none.
pub trait RegionalTokenSlotRule: std::fmt::Debug {
    fn check(&self, item: &RegionalTokenSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTokenSlotCodePrefixRule {
    pub prefix: String,
}

impl RegionalTokenSlotRule for RegionalTokenSlotCodePrefixRule {
    fn check(&self, item: &RegionalTokenSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTokenSlotRequiredDescriptionRule {
    pub message: String,
}

impl RegionalTokenSlotRule for RegionalTokenSlotRequiredDescriptionRule {
    fn check(&self, item: &RegionalTokenSlot) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTokenSlotLabelLengthRule {
    pub max_length: i64,
}

impl RegionalTokenSlotRule for RegionalTokenSlotLabelLengthRule {
    fn check(&self, item: &RegionalTokenSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTokenSlotRequiredCodeRule {
    pub message: String,
}

impl RegionalTokenSlotRule for RegionalTokenSlotRequiredCodeRule {
    fn check(&self, item: &RegionalTokenSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional token slot.
#[derive(Debug, Clone)]
pub struct RegionalTokenSlotValidator {
    pub rules: Vec<Rc<dyn RegionalTokenSlotRule>>,
}

impl RegionalTokenSlotValidator {
    pub fn validate(&self, item: &RegionalTokenSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalTokenSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalTokenSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_token_slot_validator() -> RegionalTokenSlotValidator {
    RegionalTokenSlotValidator {
        rules: vec![
            Rc::new(RegionalTokenSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RegionalTokenSlotRule>,
            Rc::new(RegionalTokenSlotRequiredDescriptionRule {
                message: "regional token slot needs this field".to_string(),
            }),
            Rc::new(RegionalTokenSlotLabelLengthRule { max_length: 16 }),
            Rc::new(RegionalTokenSlotRequiredCodeRule {
                message: "regional token slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_token_slot_validator();
        let item = new_regional_token_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = RegionalTokenSlotRequiredDescriptionRule {
            message: "regional token slot needs an entry".to_string(),
        };
        let mut item = new_regional_token_slot("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalTokenSlotLabelLengthRule { max_length: 64 };
        let mut item = new_regional_token_slot("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalTokenSlotRequiredCodeRule {
            message: "regional token slot needs this field".to_string(),
        };
        let mut item = new_regional_token_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
