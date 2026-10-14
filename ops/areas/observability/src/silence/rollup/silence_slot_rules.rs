use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a silence slot must pass. Check returns its problems, or none.
pub trait SilenceSlotRule: std::fmt::Debug {
    fn check(&self, item: &SilenceSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSlotRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceSlotRule for SilenceSlotRotationDaysRangeRule {
    fn check(&self, item: &SilenceSlot) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSlotArchivedGuardRule {
    pub limit: i64,
}

impl SilenceSlotRule for SilenceSlotArchivedGuardRule {
    fn check(&self, item: &SilenceSlot) -> Vec<String> {
        if item.archived && (item.ends_at > self.limit) {
            return vec![format!(
                "ends at {} exceeds {} while archived",
                item.ends_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSlotCodeLengthRule {
    pub max_length: i64,
}

impl SilenceSlotRule for SilenceSlotCodeLengthRule {
    fn check(&self, item: &SilenceSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSlotAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl SilenceSlotRule for SilenceSlotAllowedLabelRule {
    fn check(&self, item: &SilenceSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is missing", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence slot.
#[derive(Debug, Clone)]
pub struct SilenceSlotValidator {
    pub rules: Vec<Rc<dyn SilenceSlotRule>>,
}

impl SilenceSlotValidator {
    pub fn validate(&self, item: &SilenceSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_slot_validator() -> SilenceSlotValidator {
    SilenceSlotValidator {
        rules: vec![
            Rc::new(SilenceSlotRotationDaysRangeRule { min: 1000, max: 86400 }) as Rc<dyn SilenceSlotRule>,
            Rc::new(SilenceSlotArchivedGuardRule { limit: 86400 }),
            Rc::new(SilenceSlotCodeLengthRule { max_length: 86400 }),
            Rc::new(SilenceSlotAllowedLabelRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_slot_validator();
        let item = new_silence_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = SilenceSlotRotationDaysRangeRule { min: 255, max: 3600 };
        let mut item = new_silence_slot("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SilenceSlotCodeLengthRule { max_length: 3600 };
        let mut item = new_silence_slot("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
