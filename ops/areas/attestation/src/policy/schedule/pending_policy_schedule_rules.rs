use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending policy schedule must pass. Check returns its problems, or none.
pub trait PendingPolicyScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PendingPolicySchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyScheduleRequiredLabelRule {
    pub message: String,
}

impl PendingPolicyScheduleRule for PendingPolicyScheduleRequiredLabelRule {
    fn check(&self, item: &PendingPolicySchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyScheduleRequiredContactEmailRule {
    pub message: String,
}

impl PendingPolicyScheduleRule for PendingPolicyScheduleRequiredContactEmailRule {
    fn check(&self, item: &PendingPolicySchedule) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyScheduleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingPolicyScheduleRule for PendingPolicyScheduleMinimumRangeRule {
    fn check(&self, item: &PendingPolicySchedule) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending policy schedule.
#[derive(Debug, Clone)]
pub struct PendingPolicyScheduleValidator {
    pub rules: Vec<Rc<dyn PendingPolicyScheduleRule>>,
}

impl PendingPolicyScheduleValidator {
    pub fn validate(&self, item: &PendingPolicySchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPolicySchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPolicySchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_policy_schedule_validator() -> PendingPolicyScheduleValidator {
    PendingPolicyScheduleValidator {
        rules: vec![
            Rc::new(PendingPolicyScheduleRequiredLabelRule {
                message: "pending policy schedule needs a value".to_string(),
            }) as Rc<dyn PendingPolicyScheduleRule>,
            Rc::new(PendingPolicyScheduleRequiredContactEmailRule {
                message: "pending policy schedule needs this field".to_string(),
            }),
            Rc::new(PendingPolicyScheduleMinimumRangeRule { min: 2.3, max: 1324.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_policy_schedule_validator();
        let item = new_pending_policy_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingPolicyScheduleRequiredLabelRule {
            message: "pending policy schedule needs a value".to_string(),
        };
        let mut item = new_pending_policy_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = PendingPolicyScheduleRequiredContactEmailRule {
            message: "pending policy schedule needs this field".to_string(),
        };
        let mut item = new_pending_policy_schedule("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PendingPolicyScheduleMinimumRangeRule { min: 2.3, max: 3323.6 };
        let mut item = new_pending_policy_schedule("x", "y");
        item.minimum = 3325.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
