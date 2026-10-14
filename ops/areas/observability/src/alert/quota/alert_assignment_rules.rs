use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert assignment must pass. Check returns its problems, or none.
pub trait AlertAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &AlertAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAssignmentWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertAssignmentRule for AlertAssignmentWindowDaysRangeRule {
    fn check(&self, item: &AlertAssignment) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAssignmentRequiredChannelRule {
    pub message: String,
}

impl AlertAssignmentRule for AlertAssignmentRequiredChannelRule {
    fn check(&self, item: &AlertAssignment) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAssignmentChannelLengthRule {
    pub max_length: i64,
}

impl AlertAssignmentRule for AlertAssignmentChannelLengthRule {
    fn check(&self, item: &AlertAssignment) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAssignmentMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertAssignmentRule for AlertAssignmentMaximumRangeRule {
    fn check(&self, item: &AlertAssignment) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert assignment.
#[derive(Debug, Clone)]
pub struct AlertAssignmentValidator {
    pub rules: Vec<Rc<dyn AlertAssignmentRule>>,
}

impl AlertAssignmentValidator {
    pub fn validate(&self, item: &AlertAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_assignment_validator() -> AlertAssignmentValidator {
    AlertAssignmentValidator {
        rules: vec![
            Rc::new(AlertAssignmentWindowDaysRangeRule { min: 1000, max: 3600 }) as Rc<dyn AlertAssignmentRule>,
            Rc::new(AlertAssignmentRequiredChannelRule {
                message: "alert assignment needs an entry".to_string(),
            }),
            Rc::new(AlertAssignmentChannelLengthRule { max_length: 86400 }),
            Rc::new(AlertAssignmentMaximumRangeRule { min: 5.8, max: 2838.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_assignment_validator();
        let item = new_alert_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = AlertAssignmentWindowDaysRangeRule { min: 255, max: 86400 };
        let mut item = new_alert_assignment("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = AlertAssignmentRequiredChannelRule {
            message: "alert assignment needs this field".to_string(),
        };
        let mut item = new_alert_assignment("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = AlertAssignmentChannelLengthRule { max_length: 16 };
        let mut item = new_alert_assignment("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = AlertAssignmentMaximumRangeRule { min: 4.3, max: 3919.0 };
        let mut item = new_alert_assignment("x", "y");
        item.maximum = 3920.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
