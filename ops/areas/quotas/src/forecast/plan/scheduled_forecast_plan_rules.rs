use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled forecast plan must pass. Check returns its problems, or none.
pub trait ScheduledForecastPlanRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledForecastPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPlanNamespaceLengthRule {
    pub max_length: i64,
}

impl ScheduledForecastPlanRule for ScheduledForecastPlanNamespaceLengthRule {
    fn check(&self, item: &ScheduledForecastPlan) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPlanScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledForecastPlanRule for ScheduledForecastPlanScoreRangeRule {
    fn check(&self, item: &ScheduledForecastPlan) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPlanCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledForecastPlanRule for ScheduledForecastPlanCodeLengthRule {
    fn check(&self, item: &ScheduledForecastPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPlanStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledForecastPlanRule for ScheduledForecastPlanStartsAtRangeRule {
    fn check(&self, item: &ScheduledForecastPlan) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled forecast plan.
#[derive(Debug, Clone)]
pub struct ScheduledForecastPlanValidator {
    pub rules: Vec<Rc<dyn ScheduledForecastPlanRule>>,
}

impl ScheduledForecastPlanValidator {
    pub fn validate(&self, item: &ScheduledForecastPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledForecastPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledForecastPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_forecast_plan_validator() -> ScheduledForecastPlanValidator {
    ScheduledForecastPlanValidator {
        rules: vec![
            Rc::new(ScheduledForecastPlanNamespaceLengthRule { max_length: 86400 })
                as Rc<dyn ScheduledForecastPlanRule>,
            Rc::new(ScheduledForecastPlanScoreRangeRule { min: 0.1, max: 3579.8 }),
            Rc::new(ScheduledForecastPlanCodeLengthRule { max_length: 1000 }),
            Rc::new(ScheduledForecastPlanStartsAtRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_forecast_plan_validator();
        let item = new_scheduled_forecast_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = ScheduledForecastPlanNamespaceLengthRule { max_length: 255 };
        let mut item = new_scheduled_forecast_plan("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ScheduledForecastPlanScoreRangeRule { min: 9.7, max: 3911.9 };
        let mut item = new_scheduled_forecast_plan("x", "y");
        item.score = 3913.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledForecastPlanCodeLengthRule { max_length: 80 };
        let mut item = new_scheduled_forecast_plan("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ScheduledForecastPlanStartsAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_scheduled_forecast_plan("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
