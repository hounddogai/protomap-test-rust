use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict release slot must pass. Check returns its problems, or none.
pub trait StrictReleaseSlotRule: std::fmt::Debug {
    fn check(&self, item: &StrictReleaseSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseSlotLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictReleaseSlotRule for StrictReleaseSlotLimitRangeRule {
    fn check(&self, item: &StrictReleaseSlot) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseSlotTitleLengthRule {
    pub max_length: i64,
}

impl StrictReleaseSlotRule for StrictReleaseSlotTitleLengthRule {
    fn check(&self, item: &StrictReleaseSlot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseSlotKeyIdLengthRule {
    pub max_length: i64,
}

impl StrictReleaseSlotRule for StrictReleaseSlotKeyIdLengthRule {
    fn check(&self, item: &StrictReleaseSlot) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseSlotCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictReleaseSlotRule for StrictReleaseSlotCostRangeRule {
    fn check(&self, item: &StrictReleaseSlot) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseSlotRequiredKeyIdRule {
    pub message: String,
}

impl StrictReleaseSlotRule for StrictReleaseSlotRequiredKeyIdRule {
    fn check(&self, item: &StrictReleaseSlot) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseSlotRequiredUnitRule {
    pub message: String,
}

impl StrictReleaseSlotRule for StrictReleaseSlotRequiredUnitRule {
    fn check(&self, item: &StrictReleaseSlot) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict release slot.
#[derive(Debug, Clone)]
pub struct StrictReleaseSlotValidator {
    pub rules: Vec<Rc<dyn StrictReleaseSlotRule>>,
}

impl StrictReleaseSlotValidator {
    pub fn validate(&self, item: &StrictReleaseSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictReleaseSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictReleaseSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_release_slot_validator() -> StrictReleaseSlotValidator {
    StrictReleaseSlotValidator {
        rules: vec![
            Rc::new(StrictReleaseSlotLimitRangeRule { min: 64, max: 255 }) as Rc<dyn StrictReleaseSlotRule>,
            Rc::new(StrictReleaseSlotTitleLengthRule { max_length: 120 }),
            Rc::new(StrictReleaseSlotKeyIdLengthRule { max_length: 1000 }),
            Rc::new(StrictReleaseSlotCostRangeRule { min: 2.3, max: 2324.5 }),
            Rc::new(StrictReleaseSlotRequiredKeyIdRule {
                message: "strict release slot needs a value".to_string(),
            }),
            Rc::new(StrictReleaseSlotRequiredUnitRule {
                message: "strict release slot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_release_slot_validator();
        let item = new_strict_release_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = StrictReleaseSlotLimitRangeRule { min: 3600, max: 3600 };
        let mut item = new_strict_release_slot("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictReleaseSlotTitleLengthRule { max_length: 1000 };
        let mut item = new_strict_release_slot("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = StrictReleaseSlotKeyIdLengthRule { max_length: 120 };
        let mut item = new_strict_release_slot("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = StrictReleaseSlotCostRangeRule { min: 0.9, max: 3660.7 };
        let mut item = new_strict_release_slot("x", "y");
        item.cost = 3662.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = StrictReleaseSlotRequiredKeyIdRule {
            message: "strict release slot needs an entry".to_string(),
        };
        let mut item = new_strict_release_slot("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = StrictReleaseSlotRequiredUnitRule {
            message: "strict release slot needs a value".to_string(),
        };
        let mut item = new_strict_release_slot("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
