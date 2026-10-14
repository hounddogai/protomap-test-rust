use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary schedule must pass. Check returns its problems, or none.
pub trait CanaryScheduleRule: std::fmt::Debug {
    fn check(&self, item: &CanarySchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScheduleAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl CanaryScheduleRule for CanaryScheduleAllowedCipherRule {
    fn check(&self, item: &CanarySchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is not allowed", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScheduleRequiredCodeRule {
    pub message: String,
}

impl CanaryScheduleRule for CanaryScheduleRequiredCodeRule {
    fn check(&self, item: &CanarySchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScheduleCodeLengthRule {
    pub max_length: i64,
}

impl CanaryScheduleRule for CanaryScheduleCodeLengthRule {
    fn check(&self, item: &CanarySchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScheduleDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryScheduleRule for CanaryScheduleDurationMinutesRangeRule {
    fn check(&self, item: &CanarySchedule) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScheduleCipherLengthRule {
    pub max_length: i64,
}

impl CanaryScheduleRule for CanaryScheduleCipherLengthRule {
    fn check(&self, item: &CanarySchedule) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScheduleLabelLengthRule {
    pub max_length: i64,
}

impl CanaryScheduleRule for CanaryScheduleLabelLengthRule {
    fn check(&self, item: &CanarySchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary schedule.
#[derive(Debug, Clone)]
pub struct CanaryScheduleValidator {
    pub rules: Vec<Rc<dyn CanaryScheduleRule>>,
}

impl CanaryScheduleValidator {
    pub fn validate(&self, item: &CanarySchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanarySchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanarySchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_schedule_validator() -> CanaryScheduleValidator {
    CanaryScheduleValidator {
        rules: vec![
            Rc::new(CanaryScheduleAllowedCipherRule {
                allowed: vec!["weighted".to_string(), "strict".to_string(), "global".to_string()],
            }) as Rc<dyn CanaryScheduleRule>,
            Rc::new(CanaryScheduleRequiredCodeRule {
                message: "canary schedule needs this field".to_string(),
            }),
            Rc::new(CanaryScheduleCodeLengthRule { max_length: 3600 }),
            Rc::new(CanaryScheduleDurationMinutesRangeRule { min: 80, max: 3600 }),
            Rc::new(CanaryScheduleCipherLengthRule { max_length: 255 }),
            Rc::new(CanaryScheduleLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_schedule_validator();
        let item = new_canary_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryScheduleRequiredCodeRule {
            message: "canary schedule needs an entry".to_string(),
        };
        let mut item = new_canary_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanaryScheduleCodeLengthRule { max_length: 1000 };
        let mut item = new_canary_schedule("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = CanaryScheduleDurationMinutesRangeRule { min: 16, max: 86400 };
        let mut item = new_canary_schedule("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = CanaryScheduleCipherLengthRule { max_length: 32 };
        let mut item = new_canary_schedule("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CanaryScheduleLabelLengthRule { max_length: 86400 };
        let mut item = new_canary_schedule("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
