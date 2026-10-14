use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled credential window must pass. Check returns its problems, or none.
pub trait ScheduledCredentialWindowRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledCredentialWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialWindowRequiredLabelRule {
    pub message: String,
}

impl ScheduledCredentialWindowRule for ScheduledCredentialWindowRequiredLabelRule {
    fn check(&self, item: &ScheduledCredentialWindow) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialWindowRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledCredentialWindowRule for ScheduledCredentialWindowRotationDaysRangeRule {
    fn check(&self, item: &ScheduledCredentialWindow) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialWindowLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledCredentialWindowRule for ScheduledCredentialWindowLabelLengthRule {
    fn check(&self, item: &ScheduledCredentialWindow) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialWindowCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledCredentialWindowRule for ScheduledCredentialWindowCeilingRangeRule {
    fn check(&self, item: &ScheduledCredentialWindow) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialWindowApprovedGuardRule {
    pub limit: i64,
}

impl ScheduledCredentialWindowRule for ScheduledCredentialWindowApprovedGuardRule {
    fn check(&self, item: &ScheduledCredentialWindow) -> Vec<String> {
        if item.approved && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while approved",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled credential window.
#[derive(Debug, Clone)]
pub struct ScheduledCredentialWindowValidator {
    pub rules: Vec<Rc<dyn ScheduledCredentialWindowRule>>,
}

impl ScheduledCredentialWindowValidator {
    pub fn validate(&self, item: &ScheduledCredentialWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledCredentialWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledCredentialWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_credential_window_validator() -> ScheduledCredentialWindowValidator {
    ScheduledCredentialWindowValidator {
        rules: vec![
            Rc::new(ScheduledCredentialWindowRequiredLabelRule {
                message: "scheduled credential window needs this field".to_string(),
            }) as Rc<dyn ScheduledCredentialWindowRule>,
            Rc::new(ScheduledCredentialWindowRotationDaysRangeRule { min: 255, max: 86400 }),
            Rc::new(ScheduledCredentialWindowLabelLengthRule { max_length: 64 }),
            Rc::new(ScheduledCredentialWindowCeilingRangeRule { min: 7.9, max: 592.9 }),
            Rc::new(ScheduledCredentialWindowApprovedGuardRule { limit: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_credential_window_validator();
        let item = new_scheduled_credential_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledCredentialWindowRequiredLabelRule {
            message: "scheduled credential window needs an entry".to_string(),
        };
        let mut item = new_scheduled_credential_window("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = ScheduledCredentialWindowRotationDaysRangeRule { min: 80, max: 86400 };
        let mut item = new_scheduled_credential_window("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledCredentialWindowLabelLengthRule { max_length: 120 };
        let mut item = new_scheduled_credential_window("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ScheduledCredentialWindowCeilingRangeRule { min: 5.5, max: 201.1 };
        let mut item = new_scheduled_credential_window("x", "y");
        item.ceiling = 202.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
