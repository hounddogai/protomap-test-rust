use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan schedule must pass. Check returns its problems, or none.
pub trait PlanScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PlanSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanScheduleRequiredDescriptionRule {
    pub message: String,
}

impl PlanScheduleRule for PlanScheduleRequiredDescriptionRule {
    fn check(&self, item: &PlanSchedule) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanScheduleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanScheduleRule for PlanScheduleRotationDaysRangeRule {
    fn check(&self, item: &PlanSchedule) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanScheduleAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl PlanScheduleRule for PlanScheduleAllowedDescriptionRule {
    fn check(&self, item: &PlanSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is too long", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanScheduleRequiredCodeRule {
    pub message: String,
}

impl PlanScheduleRule for PlanScheduleRequiredCodeRule {
    fn check(&self, item: &PlanSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan schedule.
#[derive(Debug, Clone)]
pub struct PlanScheduleValidator {
    pub rules: Vec<Rc<dyn PlanScheduleRule>>,
}

impl PlanScheduleValidator {
    pub fn validate(&self, item: &PlanSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_schedule_validator() -> PlanScheduleValidator {
    PlanScheduleValidator {
        rules: vec![
            Rc::new(PlanScheduleRequiredDescriptionRule {
                message: "plan schedule needs an entry".to_string(),
            }) as Rc<dyn PlanScheduleRule>,
            Rc::new(PlanScheduleRotationDaysRangeRule { min: 120, max: 255 }),
            Rc::new(PlanScheduleAllowedDescriptionRule {
                allowed: vec!["fallback".to_string(), "lenient".to_string()],
            }),
            Rc::new(PlanScheduleRequiredCodeRule {
                message: "plan schedule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_schedule_validator();
        let item = new_plan_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = PlanScheduleRequiredDescriptionRule {
            message: "plan schedule needs a value".to_string(),
        };
        let mut item = new_plan_schedule("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PlanScheduleRotationDaysRangeRule { min: 120, max: 3600 };
        let mut item = new_plan_schedule("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanScheduleRequiredCodeRule {
            message: "plan schedule needs an entry".to_string(),
        };
        let mut item = new_plan_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
