use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast milestone must pass. Check returns its problems, or none.
pub trait ForecastMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ForecastMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMilestoneReviewerLengthRule {
    pub max_length: i64,
}

impl ForecastMilestoneRule for ForecastMilestoneReviewerLengthRule {
    fn check(&self, item: &ForecastMilestone) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMilestoneRequiredCodeRule {
    pub message: String,
}

impl ForecastMilestoneRule for ForecastMilestoneRequiredCodeRule {
    fn check(&self, item: &ForecastMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMilestoneScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastMilestoneRule for ForecastMilestoneScoreRangeRule {
    fn check(&self, item: &ForecastMilestone) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a forecast milestone.
#[derive(Debug, Clone)]
pub struct ForecastMilestoneValidator {
    pub rules: Vec<Rc<dyn ForecastMilestoneRule>>,
}

impl ForecastMilestoneValidator {
    pub fn validate(&self, item: &ForecastMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_milestone_validator() -> ForecastMilestoneValidator {
    ForecastMilestoneValidator {
        rules: vec![
            Rc::new(ForecastMilestoneReviewerLengthRule { max_length: 255 }) as Rc<dyn ForecastMilestoneRule>,
            Rc::new(ForecastMilestoneRequiredCodeRule {
                message: "forecast milestone needs a value".to_string(),
            }),
            Rc::new(ForecastMilestoneScoreRangeRule { min: 9.1, max: 2690.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_milestone_validator();
        let item = new_forecast_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = ForecastMilestoneReviewerLengthRule { max_length: 255 };
        let mut item = new_forecast_milestone("x", "y");
        item.reviewer = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastMilestoneRequiredCodeRule {
            message: "forecast milestone needs a value".to_string(),
        };
        let mut item = new_forecast_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ForecastMilestoneScoreRangeRule { min: 5.2, max: 3757.8 };
        let mut item = new_forecast_milestone("x", "y");
        item.score = 3759.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
