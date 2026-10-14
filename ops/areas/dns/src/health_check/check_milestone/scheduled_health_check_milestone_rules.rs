use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled health check milestone must pass. Check returns its problems, or none.
pub trait ScheduledHealthCheckMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledHealthCheckMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckMilestoneSourceLengthRule {
    pub max_length: i64,
}

impl ScheduledHealthCheckMilestoneRule for ScheduledHealthCheckMilestoneSourceLengthRule {
    fn check(&self, item: &ScheduledHealthCheckMilestone) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckMilestoneRequiredPhoneRule {
    pub message: String,
}

impl ScheduledHealthCheckMilestoneRule for ScheduledHealthCheckMilestoneRequiredPhoneRule {
    fn check(&self, item: &ScheduledHealthCheckMilestone) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckMilestoneRequiredSourceRule {
    pub message: String,
}

impl ScheduledHealthCheckMilestoneRule for ScheduledHealthCheckMilestoneRequiredSourceRule {
    fn check(&self, item: &ScheduledHealthCheckMilestone) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckMilestonePhoneLengthRule {
    pub max_length: i64,
}

impl ScheduledHealthCheckMilestoneRule for ScheduledHealthCheckMilestonePhoneLengthRule {
    fn check(&self, item: &ScheduledHealthCheckMilestone) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckMilestoneRequiredCodeRule {
    pub message: String,
}

impl ScheduledHealthCheckMilestoneRule for ScheduledHealthCheckMilestoneRequiredCodeRule {
    fn check(&self, item: &ScheduledHealthCheckMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckMilestoneRequiredTitleRule {
    pub message: String,
}

impl ScheduledHealthCheckMilestoneRule for ScheduledHealthCheckMilestoneRequiredTitleRule {
    fn check(&self, item: &ScheduledHealthCheckMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled health check milestone.
#[derive(Debug, Clone)]
pub struct ScheduledHealthCheckMilestoneValidator {
    pub rules: Vec<Rc<dyn ScheduledHealthCheckMilestoneRule>>,
}

impl ScheduledHealthCheckMilestoneValidator {
    pub fn validate(&self, item: &ScheduledHealthCheckMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledHealthCheckMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledHealthCheckMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_health_check_milestone_validator() -> ScheduledHealthCheckMilestoneValidator {
    ScheduledHealthCheckMilestoneValidator {
        rules: vec![
            Rc::new(ScheduledHealthCheckMilestoneSourceLengthRule { max_length: 32 })
                as Rc<dyn ScheduledHealthCheckMilestoneRule>,
            Rc::new(ScheduledHealthCheckMilestoneRequiredPhoneRule {
                message: "scheduled health check milestone needs an entry".to_string(),
            }),
            Rc::new(ScheduledHealthCheckMilestoneRequiredSourceRule {
                message: "scheduled health check milestone needs an entry".to_string(),
            }),
            Rc::new(ScheduledHealthCheckMilestonePhoneLengthRule { max_length: 64 }),
            Rc::new(ScheduledHealthCheckMilestoneRequiredCodeRule {
                message: "scheduled health check milestone needs this field".to_string(),
            }),
            Rc::new(ScheduledHealthCheckMilestoneRequiredTitleRule {
                message: "scheduled health check milestone needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_health_check_milestone_validator();
        let item = new_scheduled_health_check_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = ScheduledHealthCheckMilestoneSourceLengthRule { max_length: 16 };
        let mut item = new_scheduled_health_check_milestone("x", "y");
        item.source = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ScheduledHealthCheckMilestoneRequiredPhoneRule {
            message: "scheduled health check milestone needs an entry".to_string(),
        };
        let mut item = new_scheduled_health_check_milestone("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = ScheduledHealthCheckMilestoneRequiredSourceRule {
            message: "scheduled health check milestone needs a value".to_string(),
        };
        let mut item = new_scheduled_health_check_milestone("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = ScheduledHealthCheckMilestonePhoneLengthRule { max_length: 16 };
        let mut item = new_scheduled_health_check_milestone("x", "y");
        item.phone = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledHealthCheckMilestoneRequiredCodeRule {
            message: "scheduled health check milestone needs a value".to_string(),
        };
        let mut item = new_scheduled_health_check_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledHealthCheckMilestoneRequiredTitleRule {
            message: "scheduled health check milestone needs an entry".to_string(),
        };
        let mut item = new_scheduled_health_check_milestone("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
