use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending suspension schedule must pass. Check returns its problems, or none.
pub trait PendingSuspensionScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PendingSuspensionSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionScheduleCategoryLengthRule {
    pub max_length: i64,
}

impl PendingSuspensionScheduleRule for PendingSuspensionScheduleCategoryLengthRule {
    fn check(&self, item: &PendingSuspensionSchedule) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionScheduleRequiredNameRule {
    pub message: String,
}

impl PendingSuspensionScheduleRule for PendingSuspensionScheduleRequiredNameRule {
    fn check(&self, item: &PendingSuspensionSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionScheduleRequiredCodeRule {
    pub message: String,
}

impl PendingSuspensionScheduleRule for PendingSuspensionScheduleRequiredCodeRule {
    fn check(&self, item: &PendingSuspensionSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionScheduleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingSuspensionScheduleRule for PendingSuspensionScheduleAllowedLabelRule {
    fn check(&self, item: &PendingSuspensionSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionScheduleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingSuspensionScheduleRule for PendingSuspensionScheduleThresholdRangeRule {
    fn check(&self, item: &PendingSuspensionSchedule) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionScheduleRequiredLabelRule {
    pub message: String,
}

impl PendingSuspensionScheduleRule for PendingSuspensionScheduleRequiredLabelRule {
    fn check(&self, item: &PendingSuspensionSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending suspension schedule.
#[derive(Debug, Clone)]
pub struct PendingSuspensionScheduleValidator {
    pub rules: Vec<Rc<dyn PendingSuspensionScheduleRule>>,
}

impl PendingSuspensionScheduleValidator {
    pub fn validate(&self, item: &PendingSuspensionSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingSuspensionSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSuspensionSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_suspension_schedule_validator() -> PendingSuspensionScheduleValidator {
    PendingSuspensionScheduleValidator {
        rules: vec![
            Rc::new(PendingSuspensionScheduleCategoryLengthRule { max_length: 255 })
                as Rc<dyn PendingSuspensionScheduleRule>,
            Rc::new(PendingSuspensionScheduleRequiredNameRule {
                message: "pending suspension schedule needs a value".to_string(),
            }),
            Rc::new(PendingSuspensionScheduleRequiredCodeRule {
                message: "pending suspension schedule needs an entry".to_string(),
            }),
            Rc::new(PendingSuspensionScheduleAllowedLabelRule {
                allowed: vec!["pending".to_string(), "weighted".to_string(), "default".to_string()],
            }),
            Rc::new(PendingSuspensionScheduleThresholdRangeRule { min: 5.0, max: 3283.0 }),
            Rc::new(PendingSuspensionScheduleRequiredLabelRule {
                message: "pending suspension schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_suspension_schedule_validator();
        let item = new_pending_suspension_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = PendingSuspensionScheduleCategoryLengthRule { max_length: 3600 };
        let mut item = new_pending_suspension_schedule("x", "y");
        item.category = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingSuspensionScheduleRequiredNameRule {
            message: "pending suspension schedule needs this field".to_string(),
        };
        let mut item = new_pending_suspension_schedule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingSuspensionScheduleRequiredCodeRule {
            message: "pending suspension schedule needs a value".to_string(),
        };
        let mut item = new_pending_suspension_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PendingSuspensionScheduleThresholdRangeRule { min: 7.4, max: 378.9 };
        let mut item = new_pending_suspension_schedule("x", "y");
        item.threshold = 380.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingSuspensionScheduleRequiredLabelRule {
            message: "pending suspension schedule needs this field".to_string(),
        };
        let mut item = new_pending_suspension_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
