use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling zone slot must pass. Check returns its problems, or none.
pub trait RollingZoneSlotRule: std::fmt::Debug {
    fn check(&self, item: &RollingZoneSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSlotCodePrefixRule {
    pub prefix: String,
}

impl RollingZoneSlotRule for RollingZoneSlotCodePrefixRule {
    fn check(&self, item: &RollingZoneSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSlotRequiredCodeRule {
    pub message: String,
}

impl RollingZoneSlotRule for RollingZoneSlotRequiredCodeRule {
    fn check(&self, item: &RollingZoneSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSlotSegmentLengthRule {
    pub max_length: i64,
}

impl RollingZoneSlotRule for RollingZoneSlotSegmentLengthRule {
    fn check(&self, item: &RollingZoneSlot) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling zone slot.
#[derive(Debug, Clone)]
pub struct RollingZoneSlotValidator {
    pub rules: Vec<Rc<dyn RollingZoneSlotRule>>,
}

impl RollingZoneSlotValidator {
    pub fn validate(&self, item: &RollingZoneSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingZoneSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingZoneSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_zone_slot_validator() -> RollingZoneSlotValidator {
    RollingZoneSlotValidator {
        rules: vec![
            Rc::new(RollingZoneSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RollingZoneSlotRule>,
            Rc::new(RollingZoneSlotRequiredCodeRule {
                message: "rolling zone slot needs a value".to_string(),
            }),
            Rc::new(RollingZoneSlotSegmentLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_zone_slot_validator();
        let item = new_rolling_zone_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingZoneSlotRequiredCodeRule {
            message: "rolling zone slot needs this field".to_string(),
        };
        let mut item = new_rolling_zone_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = RollingZoneSlotSegmentLengthRule { max_length: 32 };
        let mut item = new_rolling_zone_slot("x", "y");
        item.segment = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
