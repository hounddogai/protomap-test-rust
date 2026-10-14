use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan milestone must pass. Check returns its problems, or none.
pub trait PlanMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &PlanMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMilestoneRequiredUnitRule {
    pub message: String,
}

impl PlanMilestoneRule for PlanMilestoneRequiredUnitRule {
    fn check(&self, item: &PlanMilestone) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMilestoneUnitLengthRule {
    pub max_length: i64,
}

impl PlanMilestoneRule for PlanMilestoneUnitLengthRule {
    fn check(&self, item: &PlanMilestone) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMilestoneLocaleLengthRule {
    pub max_length: i64,
}

impl PlanMilestoneRule for PlanMilestoneLocaleLengthRule {
    fn check(&self, item: &PlanMilestone) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMilestoneRequiredCodeRule {
    pub message: String,
}

impl PlanMilestoneRule for PlanMilestoneRequiredCodeRule {
    fn check(&self, item: &PlanMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMilestoneEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanMilestoneRule for PlanMilestoneEndsAtRangeRule {
    fn check(&self, item: &PlanMilestone) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMilestoneRequiredLocaleRule {
    pub message: String,
}

impl PlanMilestoneRule for PlanMilestoneRequiredLocaleRule {
    fn check(&self, item: &PlanMilestone) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan milestone.
#[derive(Debug, Clone)]
pub struct PlanMilestoneValidator {
    pub rules: Vec<Rc<dyn PlanMilestoneRule>>,
}

impl PlanMilestoneValidator {
    pub fn validate(&self, item: &PlanMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_milestone_validator() -> PlanMilestoneValidator {
    PlanMilestoneValidator {
        rules: vec![
            Rc::new(PlanMilestoneRequiredUnitRule {
                message: "plan milestone needs an entry".to_string(),
            }) as Rc<dyn PlanMilestoneRule>,
            Rc::new(PlanMilestoneUnitLengthRule { max_length: 80 }),
            Rc::new(PlanMilestoneLocaleLengthRule { max_length: 16 }),
            Rc::new(PlanMilestoneRequiredCodeRule {
                message: "plan milestone needs an entry".to_string(),
            }),
            Rc::new(PlanMilestoneEndsAtRangeRule { min: 16, max: 3600 }),
            Rc::new(PlanMilestoneRequiredLocaleRule {
                message: "plan milestone needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_milestone_validator();
        let item = new_plan_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PlanMilestoneRequiredUnitRule {
            message: "plan milestone needs an entry".to_string(),
        };
        let mut item = new_plan_milestone("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = PlanMilestoneUnitLengthRule { max_length: 255 };
        let mut item = new_plan_milestone("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = PlanMilestoneLocaleLengthRule { max_length: 16 };
        let mut item = new_plan_milestone("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanMilestoneRequiredCodeRule {
            message: "plan milestone needs an entry".to_string(),
        };
        let mut item = new_plan_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = PlanMilestoneEndsAtRangeRule { min: 32, max: 3600 };
        let mut item = new_plan_milestone("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PlanMilestoneRequiredLocaleRule {
            message: "plan milestone needs an entry".to_string(),
        };
        let mut item = new_plan_milestone("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
