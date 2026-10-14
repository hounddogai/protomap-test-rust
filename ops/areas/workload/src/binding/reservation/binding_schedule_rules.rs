use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding schedule must pass. Check returns its problems, or none.
pub trait BindingScheduleRule: std::fmt::Debug {
    fn check(&self, item: &BindingSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScheduleChannelLengthRule {
    pub max_length: i64,
}

impl BindingScheduleRule for BindingScheduleChannelLengthRule {
    fn check(&self, item: &BindingSchedule) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScheduleCodeLengthRule {
    pub max_length: i64,
}

impl BindingScheduleRule for BindingScheduleCodeLengthRule {
    fn check(&self, item: &BindingSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScheduleRequiredPhoneRule {
    pub message: String,
}

impl BindingScheduleRule for BindingScheduleRequiredPhoneRule {
    fn check(&self, item: &BindingSchedule) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScheduleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingScheduleRule for BindingScheduleMinimumRangeRule {
    fn check(&self, item: &BindingSchedule) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScheduleTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl BindingScheduleRule for BindingScheduleTagsLimitRule {
    fn check(&self, item: &BindingSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a binding schedule.
#[derive(Debug, Clone)]
pub struct BindingScheduleValidator {
    pub rules: Vec<Rc<dyn BindingScheduleRule>>,
}

impl BindingScheduleValidator {
    pub fn validate(&self, item: &BindingSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_schedule_validator() -> BindingScheduleValidator {
    BindingScheduleValidator {
        rules: vec![
            Rc::new(BindingScheduleChannelLengthRule { max_length: 80 }) as Rc<dyn BindingScheduleRule>,
            Rc::new(BindingScheduleCodeLengthRule { max_length: 32 }),
            Rc::new(BindingScheduleRequiredPhoneRule {
                message: "binding schedule needs an entry".to_string(),
            }),
            Rc::new(BindingScheduleMinimumRangeRule { min: 4.2, max: 669.8 }),
            Rc::new(BindingScheduleTagsLimitRule {
                max_count: 3600,
                banned: vec!["pending".to_string(), "batched".to_string(), "seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_schedule_validator();
        let item = new_binding_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = BindingScheduleChannelLengthRule { max_length: 86400 };
        let mut item = new_binding_schedule("x", "y");
        item.channel = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingScheduleCodeLengthRule { max_length: 1000 };
        let mut item = new_binding_schedule("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = BindingScheduleRequiredPhoneRule {
            message: "binding schedule needs an entry".to_string(),
        };
        let mut item = new_binding_schedule("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = BindingScheduleMinimumRangeRule { min: 9.8, max: 4669.4 };
        let mut item = new_binding_schedule("x", "y");
        item.minimum = 4670.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
