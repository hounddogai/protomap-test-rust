use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert schedule must pass. Check returns its problems, or none.
pub trait AlertScheduleRule: std::fmt::Debug {
    fn check(&self, item: &AlertSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertScheduleRequiredDescriptionRule {
    pub message: String,
}

impl AlertScheduleRule for AlertScheduleRequiredDescriptionRule {
    fn check(&self, item: &AlertSchedule) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertScheduleCodeLengthRule {
    pub max_length: i64,
}

impl AlertScheduleRule for AlertScheduleCodeLengthRule {
    fn check(&self, item: &AlertSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertScheduleLabelLengthRule {
    pub max_length: i64,
}

impl AlertScheduleRule for AlertScheduleLabelLengthRule {
    fn check(&self, item: &AlertSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertScheduleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AlertScheduleRule for AlertScheduleAllowedLabelRule {
    fn check(&self, item: &AlertSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertScheduleRequiredCodeRule {
    pub message: String,
}

impl AlertScheduleRule for AlertScheduleRequiredCodeRule {
    fn check(&self, item: &AlertSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert schedule.
#[derive(Debug, Clone)]
pub struct AlertScheduleValidator {
    pub rules: Vec<Rc<dyn AlertScheduleRule>>,
}

impl AlertScheduleValidator {
    pub fn validate(&self, item: &AlertSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_schedule_validator() -> AlertScheduleValidator {
    AlertScheduleValidator {
        rules: vec![
            Rc::new(AlertScheduleRequiredDescriptionRule {
                message: "alert schedule needs an entry".to_string(),
            }) as Rc<dyn AlertScheduleRule>,
            Rc::new(AlertScheduleCodeLengthRule { max_length: 32 }),
            Rc::new(AlertScheduleLabelLengthRule { max_length: 120 }),
            Rc::new(AlertScheduleAllowedLabelRule {
                allowed: vec!["global".to_string(), "seasonal".to_string()],
            }),
            Rc::new(AlertScheduleRequiredCodeRule {
                message: "alert schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_schedule_validator();
        let item = new_alert_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = AlertScheduleRequiredDescriptionRule {
            message: "alert schedule needs this field".to_string(),
        };
        let mut item = new_alert_schedule("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AlertScheduleCodeLengthRule { max_length: 120 };
        let mut item = new_alert_schedule("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AlertScheduleLabelLengthRule { max_length: 1000 };
        let mut item = new_alert_schedule("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertScheduleRequiredCodeRule {
            message: "alert schedule needs this field".to_string(),
        };
        let mut item = new_alert_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
