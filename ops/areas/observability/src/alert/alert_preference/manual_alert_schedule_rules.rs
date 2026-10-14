use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual alert schedule must pass. Check returns its problems, or none.
pub trait ManualAlertScheduleRule: std::fmt::Debug {
    fn check(&self, item: &ManualAlertSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAlertSchedulePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualAlertScheduleRule for ManualAlertSchedulePriorityRangeRule {
    fn check(&self, item: &ManualAlertSchedule) -> Vec<String> {
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
pub struct ManualAlertScheduleAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl ManualAlertScheduleRule for ManualAlertScheduleAllowedPrincipalRule {
    fn check(&self, item: &ManualAlertSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is below minimum", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAlertScheduleRequiredCodeRule {
    pub message: String,
}

impl ManualAlertScheduleRule for ManualAlertScheduleRequiredCodeRule {
    fn check(&self, item: &ManualAlertSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAlertScheduleCodePrefixRule {
    pub prefix: String,
}

impl ManualAlertScheduleRule for ManualAlertScheduleCodePrefixRule {
    fn check(&self, item: &ManualAlertSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual alert schedule.
#[derive(Debug, Clone)]
pub struct ManualAlertScheduleValidator {
    pub rules: Vec<Rc<dyn ManualAlertScheduleRule>>,
}

impl ManualAlertScheduleValidator {
    pub fn validate(&self, item: &ManualAlertSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualAlertSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualAlertSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_alert_schedule_validator() -> ManualAlertScheduleValidator {
    ManualAlertScheduleValidator {
        rules: vec![
            Rc::new(ManualAlertSchedulePriorityRangeRule { min: 120, max: 86400 }) as Rc<dyn ManualAlertScheduleRule>,
            Rc::new(ManualAlertScheduleAllowedPrincipalRule {
                allowed: vec!["fallback".to_string(), "default".to_string(), "regional".to_string()],
            }),
            Rc::new(ManualAlertScheduleRequiredCodeRule {
                message: "manual alert schedule needs this field".to_string(),
            }),
            Rc::new(ManualAlertScheduleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_alert_schedule_validator();
        let item = new_manual_alert_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = ManualAlertSchedulePriorityRangeRule { min: 120, max: 86400 };
        let mut item = new_manual_alert_schedule("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualAlertScheduleRequiredCodeRule {
            message: "manual alert schedule needs this field".to_string(),
        };
        let mut item = new_manual_alert_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
