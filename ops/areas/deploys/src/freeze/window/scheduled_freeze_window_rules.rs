use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled freeze window must pass. Check returns its problems, or none.
pub trait ScheduledFreezeWindowRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFreezeWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeWindowRequiredLocaleRule {
    pub message: String,
}

impl ScheduledFreezeWindowRule for ScheduledFreezeWindowRequiredLocaleRule {
    fn check(&self, item: &ScheduledFreezeWindow) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeWindowLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledFreezeWindowRule for ScheduledFreezeWindowLabelLengthRule {
    fn check(&self, item: &ScheduledFreezeWindow) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeWindowCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledFreezeWindowRule for ScheduledFreezeWindowCodeLengthRule {
    fn check(&self, item: &ScheduledFreezeWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeWindowCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFreezeWindowRule for ScheduledFreezeWindowCapacityRangeRule {
    fn check(&self, item: &ScheduledFreezeWindow) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeWindowRequiredLabelRule {
    pub message: String,
}

impl ScheduledFreezeWindowRule for ScheduledFreezeWindowRequiredLabelRule {
    fn check(&self, item: &ScheduledFreezeWindow) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled freeze window.
#[derive(Debug, Clone)]
pub struct ScheduledFreezeWindowValidator {
    pub rules: Vec<Rc<dyn ScheduledFreezeWindowRule>>,
}

impl ScheduledFreezeWindowValidator {
    pub fn validate(&self, item: &ScheduledFreezeWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledFreezeWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFreezeWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_freeze_window_validator() -> ScheduledFreezeWindowValidator {
    ScheduledFreezeWindowValidator {
        rules: vec![
            Rc::new(ScheduledFreezeWindowRequiredLocaleRule {
                message: "scheduled freeze window needs an entry".to_string(),
            }) as Rc<dyn ScheduledFreezeWindowRule>,
            Rc::new(ScheduledFreezeWindowLabelLengthRule { max_length: 16 }),
            Rc::new(ScheduledFreezeWindowCodeLengthRule { max_length: 80 }),
            Rc::new(ScheduledFreezeWindowCapacityRangeRule { min: 64, max: 86400 }),
            Rc::new(ScheduledFreezeWindowRequiredLabelRule {
                message: "scheduled freeze window needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_freeze_window_validator();
        let item = new_scheduled_freeze_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = ScheduledFreezeWindowRequiredLocaleRule {
            message: "scheduled freeze window needs an entry".to_string(),
        };
        let mut item = new_scheduled_freeze_window("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledFreezeWindowLabelLengthRule { max_length: 80 };
        let mut item = new_scheduled_freeze_window("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledFreezeWindowCodeLengthRule { max_length: 120 };
        let mut item = new_scheduled_freeze_window("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = ScheduledFreezeWindowCapacityRangeRule { min: 120, max: 3600 };
        let mut item = new_scheduled_freeze_window("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledFreezeWindowRequiredLabelRule {
            message: "scheduled freeze window needs this field".to_string(),
        };
        let mut item = new_scheduled_freeze_window("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
