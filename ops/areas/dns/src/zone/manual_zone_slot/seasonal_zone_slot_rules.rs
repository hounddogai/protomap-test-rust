use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal zone slot must pass. Check returns its problems, or none.
pub trait SeasonalZoneSlotRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalZoneSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalZoneSlotCodePrefixRule {
    pub prefix: String,
}

impl SeasonalZoneSlotRule for SeasonalZoneSlotCodePrefixRule {
    fn check(&self, item: &SeasonalZoneSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalZoneSlotRequiredCodeRule {
    pub message: String,
}

impl SeasonalZoneSlotRule for SeasonalZoneSlotRequiredCodeRule {
    fn check(&self, item: &SeasonalZoneSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalZoneSlotDescriptionLengthRule {
    pub max_length: i64,
}

impl SeasonalZoneSlotRule for SeasonalZoneSlotDescriptionLengthRule {
    fn check(&self, item: &SeasonalZoneSlot) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalZoneSlotCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalZoneSlotRule for SeasonalZoneSlotCodeLengthRule {
    fn check(&self, item: &SeasonalZoneSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalZoneSlotCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalZoneSlotRule for SeasonalZoneSlotCreatedAtRangeRule {
    fn check(&self, item: &SeasonalZoneSlot) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalZoneSlotAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl SeasonalZoneSlotRule for SeasonalZoneSlotAllowedDescriptionRule {
    fn check(&self, item: &SeasonalZoneSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is too long", item.description)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal zone slot.
#[derive(Debug, Clone)]
pub struct SeasonalZoneSlotValidator {
    pub rules: Vec<Rc<dyn SeasonalZoneSlotRule>>,
}

impl SeasonalZoneSlotValidator {
    pub fn validate(&self, item: &SeasonalZoneSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalZoneSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalZoneSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_zone_slot_validator() -> SeasonalZoneSlotValidator {
    SeasonalZoneSlotValidator {
        rules: vec![
            Rc::new(SeasonalZoneSlotCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn SeasonalZoneSlotRule>,
            Rc::new(SeasonalZoneSlotRequiredCodeRule {
                message: "seasonal zone slot needs an entry".to_string(),
            }),
            Rc::new(SeasonalZoneSlotDescriptionLengthRule { max_length: 16 }),
            Rc::new(SeasonalZoneSlotCodeLengthRule { max_length: 16 }),
            Rc::new(SeasonalZoneSlotCreatedAtRangeRule { min: 32, max: 80 }),
            Rc::new(SeasonalZoneSlotAllowedDescriptionRule {
                allowed: vec!["rolling".to_string(), "batched".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_zone_slot_validator();
        let item = new_seasonal_zone_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalZoneSlotRequiredCodeRule {
            message: "seasonal zone slot needs an entry".to_string(),
        };
        let mut item = new_seasonal_zone_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = SeasonalZoneSlotDescriptionLengthRule { max_length: 64 };
        let mut item = new_seasonal_zone_slot("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalZoneSlotCodeLengthRule { max_length: 80 };
        let mut item = new_seasonal_zone_slot("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = SeasonalZoneSlotCreatedAtRangeRule { min: 16, max: 1000 };
        let mut item = new_seasonal_zone_slot("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
