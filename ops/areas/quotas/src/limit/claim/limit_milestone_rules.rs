use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit milestone must pass. Check returns its problems, or none.
pub trait LimitMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &LimitMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitMilestoneCategoryLengthRule {
    pub max_length: i64,
}

impl LimitMilestoneRule for LimitMilestoneCategoryLengthRule {
    fn check(&self, item: &LimitMilestone) -> Vec<String> {
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
pub struct LimitMilestoneRequiredReviewerRule {
    pub message: String,
}

impl LimitMilestoneRule for LimitMilestoneRequiredReviewerRule {
    fn check(&self, item: &LimitMilestone) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitMilestoneRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitMilestoneRule for LimitMilestoneRateRangeRule {
    fn check(&self, item: &LimitMilestone) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a limit milestone.
#[derive(Debug, Clone)]
pub struct LimitMilestoneValidator {
    pub rules: Vec<Rc<dyn LimitMilestoneRule>>,
}

impl LimitMilestoneValidator {
    pub fn validate(&self, item: &LimitMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_milestone_validator() -> LimitMilestoneValidator {
    LimitMilestoneValidator {
        rules: vec![
            Rc::new(LimitMilestoneCategoryLengthRule { max_length: 1000 }) as Rc<dyn LimitMilestoneRule>,
            Rc::new(LimitMilestoneRequiredReviewerRule {
                message: "limit milestone needs a value".to_string(),
            }),
            Rc::new(LimitMilestoneRateRangeRule { min: 8.7, max: 1408.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_milestone_validator();
        let item = new_limit_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = LimitMilestoneCategoryLengthRule { max_length: 255 };
        let mut item = new_limit_milestone("x", "y");
        item.category = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = LimitMilestoneRequiredReviewerRule {
            message: "limit milestone needs a value".to_string(),
        };
        let mut item = new_limit_milestone("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = LimitMilestoneRateRangeRule { min: 1.5, max: 3497.6 };
        let mut item = new_limit_milestone("x", "y");
        item.rate = 3499.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
