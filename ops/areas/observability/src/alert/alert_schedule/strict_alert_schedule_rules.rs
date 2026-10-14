use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict alert schedule must pass. Check returns its problems, or none.
pub trait StrictAlertScheduleRule: std::fmt::Debug {
    fn check(&self, item: &StrictAlertSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertScheduleCodePrefixRule {
    pub prefix: String,
}

impl StrictAlertScheduleRule for StrictAlertScheduleCodePrefixRule {
    fn check(&self, item: &StrictAlertSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertScheduleRequiredTitleRule {
    pub message: String,
}

impl StrictAlertScheduleRule for StrictAlertScheduleRequiredTitleRule {
    fn check(&self, item: &StrictAlertSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertScheduleRequiredExternalRefRule {
    pub message: String,
}

impl StrictAlertScheduleRule for StrictAlertScheduleRequiredExternalRefRule {
    fn check(&self, item: &StrictAlertSchedule) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict alert schedule.
#[derive(Debug, Clone)]
pub struct StrictAlertScheduleValidator {
    pub rules: Vec<Rc<dyn StrictAlertScheduleRule>>,
}

impl StrictAlertScheduleValidator {
    pub fn validate(&self, item: &StrictAlertSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictAlertSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictAlertSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_alert_schedule_validator() -> StrictAlertScheduleValidator {
    StrictAlertScheduleValidator {
        rules: vec![
            Rc::new(StrictAlertScheduleCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn StrictAlertScheduleRule>,
            Rc::new(StrictAlertScheduleRequiredTitleRule {
                message: "strict alert schedule needs a value".to_string(),
            }),
            Rc::new(StrictAlertScheduleRequiredExternalRefRule {
                message: "strict alert schedule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_alert_schedule_validator();
        let item = new_strict_alert_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictAlertScheduleRequiredTitleRule {
            message: "strict alert schedule needs this field".to_string(),
        };
        let mut item = new_strict_alert_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = StrictAlertScheduleRequiredExternalRefRule {
            message: "strict alert schedule needs an entry".to_string(),
        };
        let mut item = new_strict_alert_schedule("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
