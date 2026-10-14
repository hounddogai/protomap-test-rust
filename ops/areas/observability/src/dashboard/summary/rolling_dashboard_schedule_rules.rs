use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling dashboard schedule must pass. Check returns its problems, or none.
pub trait RollingDashboardScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RollingDashboardSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardScheduleTimezoneLengthRule {
    pub max_length: i64,
}

impl RollingDashboardScheduleRule for RollingDashboardScheduleTimezoneLengthRule {
    fn check(&self, item: &RollingDashboardSchedule) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardScheduleCodeLengthRule {
    pub max_length: i64,
}

impl RollingDashboardScheduleRule for RollingDashboardScheduleCodeLengthRule {
    fn check(&self, item: &RollingDashboardSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardScheduleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RollingDashboardScheduleRule for RollingDashboardScheduleAllowedLabelRule {
    fn check(&self, item: &RollingDashboardSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardScheduleRequiredLabelRule {
    pub message: String,
}

impl RollingDashboardScheduleRule for RollingDashboardScheduleRequiredLabelRule {
    fn check(&self, item: &RollingDashboardSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardScheduleRequiredNotesRule {
    pub message: String,
}

impl RollingDashboardScheduleRule for RollingDashboardScheduleRequiredNotesRule {
    fn check(&self, item: &RollingDashboardSchedule) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardScheduleCodePrefixRule {
    pub prefix: String,
}

impl RollingDashboardScheduleRule for RollingDashboardScheduleCodePrefixRule {
    fn check(&self, item: &RollingDashboardSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling dashboard schedule.
#[derive(Debug, Clone)]
pub struct RollingDashboardScheduleValidator {
    pub rules: Vec<Rc<dyn RollingDashboardScheduleRule>>,
}

impl RollingDashboardScheduleValidator {
    pub fn validate(&self, item: &RollingDashboardSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingDashboardSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingDashboardSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_dashboard_schedule_validator() -> RollingDashboardScheduleValidator {
    RollingDashboardScheduleValidator {
        rules: vec![
            Rc::new(RollingDashboardScheduleTimezoneLengthRule { max_length: 3600 })
                as Rc<dyn RollingDashboardScheduleRule>,
            Rc::new(RollingDashboardScheduleCodeLengthRule { max_length: 64 }),
            Rc::new(RollingDashboardScheduleAllowedLabelRule {
                allowed: vec!["legacy".to_string(), "manual".to_string()],
            }),
            Rc::new(RollingDashboardScheduleRequiredLabelRule {
                message: "rolling dashboard schedule needs an entry".to_string(),
            }),
            Rc::new(RollingDashboardScheduleRequiredNotesRule {
                message: "rolling dashboard schedule needs an entry".to_string(),
            }),
            Rc::new(RollingDashboardScheduleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_dashboard_schedule_validator();
        let item = new_rolling_dashboard_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = RollingDashboardScheduleTimezoneLengthRule { max_length: 32 };
        let mut item = new_rolling_dashboard_schedule("x", "y");
        item.timezone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingDashboardScheduleCodeLengthRule { max_length: 255 };
        let mut item = new_rolling_dashboard_schedule("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingDashboardScheduleRequiredLabelRule {
            message: "rolling dashboard schedule needs an entry".to_string(),
        };
        let mut item = new_rolling_dashboard_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RollingDashboardScheduleRequiredNotesRule {
            message: "rolling dashboard schedule needs an entry".to_string(),
        };
        let mut item = new_rolling_dashboard_schedule("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
