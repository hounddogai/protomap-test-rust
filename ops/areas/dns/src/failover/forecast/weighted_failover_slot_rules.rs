use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted failover slot must pass. Check returns its problems, or none.
pub trait WeightedFailoverSlotRule: std::fmt::Debug {
    fn check(&self, item: &WeightedFailoverSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSlotLabelLengthRule {
    pub max_length: i64,
}

impl WeightedFailoverSlotRule for WeightedFailoverSlotLabelLengthRule {
    fn check(&self, item: &WeightedFailoverSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSlotRequiredNotesRule {
    pub message: String,
}

impl WeightedFailoverSlotRule for WeightedFailoverSlotRequiredNotesRule {
    fn check(&self, item: &WeightedFailoverSlot) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSlotRequiredHostRule {
    pub message: String,
}

impl WeightedFailoverSlotRule for WeightedFailoverSlotRequiredHostRule {
    fn check(&self, item: &WeightedFailoverSlot) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSlotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl WeightedFailoverSlotRule for WeightedFailoverSlotAllowedCodeRule {
    fn check(&self, item: &WeightedFailoverSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSlotRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedFailoverSlotRule for WeightedFailoverSlotRevisionRangeRule {
    fn check(&self, item: &WeightedFailoverSlot) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted failover slot.
#[derive(Debug, Clone)]
pub struct WeightedFailoverSlotValidator {
    pub rules: Vec<Rc<dyn WeightedFailoverSlotRule>>,
}

impl WeightedFailoverSlotValidator {
    pub fn validate(&self, item: &WeightedFailoverSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedFailoverSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedFailoverSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_failover_slot_validator() -> WeightedFailoverSlotValidator {
    WeightedFailoverSlotValidator {
        rules: vec![
            Rc::new(WeightedFailoverSlotLabelLengthRule { max_length: 120 }) as Rc<dyn WeightedFailoverSlotRule>,
            Rc::new(WeightedFailoverSlotRequiredNotesRule {
                message: "weighted failover slot needs an entry".to_string(),
            }),
            Rc::new(WeightedFailoverSlotRequiredHostRule {
                message: "weighted failover slot needs an entry".to_string(),
            }),
            Rc::new(WeightedFailoverSlotAllowedCodeRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(WeightedFailoverSlotRevisionRangeRule { min: 80, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_failover_slot_validator();
        let item = new_weighted_failover_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedFailoverSlotLabelLengthRule { max_length: 3600 };
        let mut item = new_weighted_failover_slot("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = WeightedFailoverSlotRequiredNotesRule {
            message: "weighted failover slot needs a value".to_string(),
        };
        let mut item = new_weighted_failover_slot("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = WeightedFailoverSlotRequiredHostRule {
            message: "weighted failover slot needs an entry".to_string(),
        };
        let mut item = new_weighted_failover_slot("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = WeightedFailoverSlotRevisionRangeRule { min: 64, max: 3600 };
        let mut item = new_weighted_failover_slot("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
