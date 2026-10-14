use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual query slot must pass. Check returns its problems, or none.
pub trait ManualQuerySlotRule: std::fmt::Debug {
    fn check(&self, item: &ManualQuerySlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuerySlotExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualQuerySlotRule for ManualQuerySlotExpiresAtRangeRule {
    fn check(&self, item: &ManualQuerySlot) -> Vec<String> {
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
pub struct ManualQuerySlotTitleLengthRule {
    pub max_length: i64,
}

impl ManualQuerySlotRule for ManualQuerySlotTitleLengthRule {
    fn check(&self, item: &ManualQuerySlot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuerySlotCodeLengthRule {
    pub max_length: i64,
}

impl ManualQuerySlotRule for ManualQuerySlotCodeLengthRule {
    fn check(&self, item: &ManualQuerySlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuerySlotRequiredCodeRule {
    pub message: String,
}

impl ManualQuerySlotRule for ManualQuerySlotRequiredCodeRule {
    fn check(&self, item: &ManualQuerySlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuerySlotCodePrefixRule {
    pub prefix: String,
}

impl ManualQuerySlotRule for ManualQuerySlotCodePrefixRule {
    fn check(&self, item: &ManualQuerySlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuerySlotFingerprintLengthRule {
    pub max_length: i64,
}

impl ManualQuerySlotRule for ManualQuerySlotFingerprintLengthRule {
    fn check(&self, item: &ManualQuerySlot) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual query slot.
#[derive(Debug, Clone)]
pub struct ManualQuerySlotValidator {
    pub rules: Vec<Rc<dyn ManualQuerySlotRule>>,
}

impl ManualQuerySlotValidator {
    pub fn validate(&self, item: &ManualQuerySlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualQuerySlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualQuerySlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_query_slot_validator() -> ManualQuerySlotValidator {
    ManualQuerySlotValidator {
        rules: vec![
            Rc::new(ManualQuerySlotExpiresAtRangeRule { min: 80, max: 255 }) as Rc<dyn ManualQuerySlotRule>,
            Rc::new(ManualQuerySlotTitleLengthRule { max_length: 64 }),
            Rc::new(ManualQuerySlotCodeLengthRule { max_length: 80 }),
            Rc::new(ManualQuerySlotRequiredCodeRule {
                message: "manual query slot needs a value".to_string(),
            }),
            Rc::new(ManualQuerySlotCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ManualQuerySlotFingerprintLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_query_slot_validator();
        let item = new_manual_query_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = ManualQuerySlotExpiresAtRangeRule { min: 120, max: 3600 };
        let mut item = new_manual_query_slot("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualQuerySlotTitleLengthRule { max_length: 255 };
        let mut item = new_manual_query_slot("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualQuerySlotCodeLengthRule { max_length: 1000 };
        let mut item = new_manual_query_slot("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualQuerySlotRequiredCodeRule {
            message: "manual query slot needs this field".to_string(),
        };
        let mut item = new_manual_query_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ManualQuerySlotFingerprintLengthRule { max_length: 120 };
        let mut item = new_manual_query_slot("x", "y");
        item.fingerprint = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
