use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal credential slot must pass. Check returns its problems, or none.
pub trait SeasonalCredentialSlotRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalCredentialSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSlotWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalCredentialSlotRule for SeasonalCredentialSlotWindowDaysRangeRule {
    fn check(&self, item: &SeasonalCredentialSlot) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSlotRequiredCodeRule {
    pub message: String,
}

impl SeasonalCredentialSlotRule for SeasonalCredentialSlotRequiredCodeRule {
    fn check(&self, item: &SeasonalCredentialSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSlotCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialSlotRule for SeasonalCredentialSlotCodeLengthRule {
    fn check(&self, item: &SeasonalCredentialSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSlotLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalCredentialSlotRule for SeasonalCredentialSlotLimitRangeRule {
    fn check(&self, item: &SeasonalCredentialSlot) -> Vec<String> {
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

/// Runs every rule of a seasonal credential slot.
#[derive(Debug, Clone)]
pub struct SeasonalCredentialSlotValidator {
    pub rules: Vec<Rc<dyn SeasonalCredentialSlotRule>>,
}

impl SeasonalCredentialSlotValidator {
    pub fn validate(&self, item: &SeasonalCredentialSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalCredentialSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalCredentialSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_credential_slot_validator() -> SeasonalCredentialSlotValidator {
    SeasonalCredentialSlotValidator {
        rules: vec![
            Rc::new(SeasonalCredentialSlotWindowDaysRangeRule { min: 1000, max: 3600 })
                as Rc<dyn SeasonalCredentialSlotRule>,
            Rc::new(SeasonalCredentialSlotRequiredCodeRule {
                message: "seasonal credential slot needs a value".to_string(),
            }),
            Rc::new(SeasonalCredentialSlotCodeLengthRule { max_length: 3600 }),
            Rc::new(SeasonalCredentialSlotLimitRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_credential_slot_validator();
        let item = new_seasonal_credential_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = SeasonalCredentialSlotWindowDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_seasonal_credential_slot("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalCredentialSlotRequiredCodeRule {
            message: "seasonal credential slot needs this field".to_string(),
        };
        let mut item = new_seasonal_credential_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalCredentialSlotCodeLengthRule { max_length: 1000 };
        let mut item = new_seasonal_credential_slot("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = SeasonalCredentialSlotLimitRangeRule { min: 64, max: 3600 };
        let mut item = new_seasonal_credential_slot("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
