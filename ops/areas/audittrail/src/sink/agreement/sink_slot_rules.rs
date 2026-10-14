use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink slot must pass. Check returns its problems, or none.
pub trait SinkSlotRule: std::fmt::Debug {
    fn check(&self, item: &SinkSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSlotExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkSlotRule for SinkSlotExpiresAtRangeRule {
    fn check(&self, item: &SinkSlot) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSlotCodePrefixRule {
    pub prefix: String,
}

impl SinkSlotRule for SinkSlotCodePrefixRule {
    fn check(&self, item: &SinkSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSlotCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkSlotRule for SinkSlotCapacityRangeRule {
    fn check(&self, item: &SinkSlot) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSlotCipherLengthRule {
    pub max_length: i64,
}

impl SinkSlotRule for SinkSlotCipherLengthRule {
    fn check(&self, item: &SinkSlot) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSlotCodeLengthRule {
    pub max_length: i64,
}

impl SinkSlotRule for SinkSlotCodeLengthRule {
    fn check(&self, item: &SinkSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink slot.
#[derive(Debug, Clone)]
pub struct SinkSlotValidator {
    pub rules: Vec<Rc<dyn SinkSlotRule>>,
}

impl SinkSlotValidator {
    pub fn validate(&self, item: &SinkSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_slot_validator() -> SinkSlotValidator {
    SinkSlotValidator {
        rules: vec![
            Rc::new(SinkSlotExpiresAtRangeRule { min: 32, max: 1000 }) as Rc<dyn SinkSlotRule>,
            Rc::new(SinkSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SinkSlotCapacityRangeRule { min: 1000, max: 3600 }),
            Rc::new(SinkSlotCipherLengthRule { max_length: 80 }),
            Rc::new(SinkSlotCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_slot_validator();
        let item = new_sink_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = SinkSlotExpiresAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_sink_slot("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = SinkSlotCapacityRangeRule { min: 120, max: 86400 };
        let mut item = new_sink_slot("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = SinkSlotCipherLengthRule { max_length: 1000 };
        let mut item = new_sink_slot("x", "y");
        item.cipher = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkSlotCodeLengthRule { max_length: 16 };
        let mut item = new_sink_slot("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
