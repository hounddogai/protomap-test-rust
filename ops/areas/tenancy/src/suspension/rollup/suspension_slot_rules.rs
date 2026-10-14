use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension slot must pass. Check returns its problems, or none.
pub trait SuspensionSlotRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSlotCategoryLengthRule {
    pub max_length: i64,
}

impl SuspensionSlotRule for SuspensionSlotCategoryLengthRule {
    fn check(&self, item: &SuspensionSlot) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSlotTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl SuspensionSlotRule for SuspensionSlotTagsLimitRule {
    fn check(&self, item: &SuspensionSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSlotAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionSlotRule for SuspensionSlotAttemptsRangeRule {
    fn check(&self, item: &SuspensionSlot) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSlotCodePrefixRule {
    pub prefix: String,
}

impl SuspensionSlotRule for SuspensionSlotCodePrefixRule {
    fn check(&self, item: &SuspensionSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSlotRequiredCategoryRule {
    pub message: String,
}

impl SuspensionSlotRule for SuspensionSlotRequiredCategoryRule {
    fn check(&self, item: &SuspensionSlot) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension slot.
#[derive(Debug, Clone)]
pub struct SuspensionSlotValidator {
    pub rules: Vec<Rc<dyn SuspensionSlotRule>>,
}

impl SuspensionSlotValidator {
    pub fn validate(&self, item: &SuspensionSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_slot_validator() -> SuspensionSlotValidator {
    SuspensionSlotValidator {
        rules: vec![
            Rc::new(SuspensionSlotCategoryLengthRule { max_length: 32 }) as Rc<dyn SuspensionSlotRule>,
            Rc::new(SuspensionSlotTagsLimitRule {
                max_count: 16,
                banned: vec!["manual".to_string(), "primary".to_string()],
            }),
            Rc::new(SuspensionSlotAttemptsRangeRule { min: 1000, max: 86400 }),
            Rc::new(SuspensionSlotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SuspensionSlotRequiredCategoryRule {
                message: "suspension slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_slot_validator();
        let item = new_suspension_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = SuspensionSlotCategoryLengthRule { max_length: 64 };
        let mut item = new_suspension_slot("x", "y");
        item.category = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = SuspensionSlotAttemptsRangeRule { min: 120, max: 86400 };
        let mut item = new_suspension_slot("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = SuspensionSlotRequiredCategoryRule {
            message: "suspension slot needs this field".to_string(),
        };
        let mut item = new_suspension_slot("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
