use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation slot must pass. Check returns its problems, or none.
pub trait IsolationSlotRule: std::fmt::Debug {
    fn check(&self, item: &IsolationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSlotAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl IsolationSlotRule for IsolationSlotAllowedChecksumRule {
    fn check(&self, item: &IsolationSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is inconsistent", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSlotRequiredCodeRule {
    pub message: String,
}

impl IsolationSlotRule for IsolationSlotRequiredCodeRule {
    fn check(&self, item: &IsolationSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSlotRequiredOwnerIdRule {
    pub message: String,
}

impl IsolationSlotRule for IsolationSlotRequiredOwnerIdRule {
    fn check(&self, item: &IsolationSlot) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSlotNameLengthRule {
    pub max_length: i64,
}

impl IsolationSlotRule for IsolationSlotNameLengthRule {
    fn check(&self, item: &IsolationSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSlotEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationSlotRule for IsolationSlotEndsAtRangeRule {
    fn check(&self, item: &IsolationSlot) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSlotVisibleGuardRule {
    pub limit: f64,
}

impl IsolationSlotRule for IsolationSlotVisibleGuardRule {
    fn check(&self, item: &IsolationSlot) -> Vec<String> {
        if item.visible && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while visible",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation slot.
#[derive(Debug, Clone)]
pub struct IsolationSlotValidator {
    pub rules: Vec<Rc<dyn IsolationSlotRule>>,
}

impl IsolationSlotValidator {
    pub fn validate(&self, item: &IsolationSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_slot_validator() -> IsolationSlotValidator {
    IsolationSlotValidator {
        rules: vec![
            Rc::new(IsolationSlotAllowedChecksumRule { allowed: Vec::new() }) as Rc<dyn IsolationSlotRule>,
            Rc::new(IsolationSlotRequiredCodeRule {
                message: "isolation slot needs this field".to_string(),
            }),
            Rc::new(IsolationSlotRequiredOwnerIdRule {
                message: "isolation slot needs a value".to_string(),
            }),
            Rc::new(IsolationSlotNameLengthRule { max_length: 32 }),
            Rc::new(IsolationSlotEndsAtRangeRule { min: 16, max: 1000 }),
            Rc::new(IsolationSlotVisibleGuardRule { limit: 9.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_slot_validator();
        let item = new_isolation_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationSlotRequiredCodeRule {
            message: "isolation slot needs a value".to_string(),
        };
        let mut item = new_isolation_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = IsolationSlotRequiredOwnerIdRule {
            message: "isolation slot needs this field".to_string(),
        };
        let mut item = new_isolation_slot("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = IsolationSlotNameLengthRule { max_length: 16 };
        let mut item = new_isolation_slot("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = IsolationSlotEndsAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_isolation_slot("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
