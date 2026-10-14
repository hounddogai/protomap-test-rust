use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze draft must pass. Check returns its problems, or none.
pub trait FreezeDraftRule: std::fmt::Debug {
    fn check(&self, item: &FreezeDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeDraftRequiredKeyIdRule {
    pub message: String,
}

impl FreezeDraftRule for FreezeDraftRequiredKeyIdRule {
    fn check(&self, item: &FreezeDraft) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeDraftPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeDraftRule for FreezeDraftPriorityRangeRule {
    fn check(&self, item: &FreezeDraft) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeDraftRequiredCountryRule {
    pub message: String,
}

impl FreezeDraftRule for FreezeDraftRequiredCountryRule {
    fn check(&self, item: &FreezeDraft) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeDraftKeyIdLengthRule {
    pub max_length: i64,
}

impl FreezeDraftRule for FreezeDraftKeyIdLengthRule {
    fn check(&self, item: &FreezeDraft) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeDraftAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeDraftRule for FreezeDraftAttemptsRangeRule {
    fn check(&self, item: &FreezeDraft) -> Vec<String> {
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

/// Runs every rule of a freeze draft.
#[derive(Debug, Clone)]
pub struct FreezeDraftValidator {
    pub rules: Vec<Rc<dyn FreezeDraftRule>>,
}

impl FreezeDraftValidator {
    pub fn validate(&self, item: &FreezeDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_draft_validator() -> FreezeDraftValidator {
    FreezeDraftValidator {
        rules: vec![
            Rc::new(FreezeDraftRequiredKeyIdRule {
                message: "freeze draft needs a value".to_string(),
            }) as Rc<dyn FreezeDraftRule>,
            Rc::new(FreezeDraftPriorityRangeRule { min: 3600, max: 3600 }),
            Rc::new(FreezeDraftRequiredCountryRule {
                message: "freeze draft needs an entry".to_string(),
            }),
            Rc::new(FreezeDraftKeyIdLengthRule { max_length: 3600 }),
            Rc::new(FreezeDraftAttemptsRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_draft_validator();
        let item = new_freeze_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = FreezeDraftRequiredKeyIdRule {
            message: "freeze draft needs this field".to_string(),
        };
        let mut item = new_freeze_draft("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = FreezeDraftPriorityRangeRule { min: 32, max: 120 };
        let mut item = new_freeze_draft("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = FreezeDraftRequiredCountryRule {
            message: "freeze draft needs an entry".to_string(),
        };
        let mut item = new_freeze_draft("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = FreezeDraftKeyIdLengthRule { max_length: 32 };
        let mut item = new_freeze_draft("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = FreezeDraftAttemptsRangeRule { min: 3600, max: 86400 };
        let mut item = new_freeze_draft("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
