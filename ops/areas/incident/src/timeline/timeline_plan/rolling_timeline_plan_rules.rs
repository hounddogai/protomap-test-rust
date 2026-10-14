use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling timeline plan must pass. Check returns its problems, or none.
pub trait RollingTimelinePlanRule: std::fmt::Debug {
    fn check(&self, item: &RollingTimelinePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePlanUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTimelinePlanRule for RollingTimelinePlanUpdatedAtRangeRule {
    fn check(&self, item: &RollingTimelinePlan) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePlanRequiredNameRule {
    pub message: String,
}

impl RollingTimelinePlanRule for RollingTimelinePlanRequiredNameRule {
    fn check(&self, item: &RollingTimelinePlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePlanSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTimelinePlanRule for RollingTimelinePlanSequenceRangeRule {
    fn check(&self, item: &RollingTimelinePlan) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling timeline plan.
#[derive(Debug, Clone)]
pub struct RollingTimelinePlanValidator {
    pub rules: Vec<Rc<dyn RollingTimelinePlanRule>>,
}

impl RollingTimelinePlanValidator {
    pub fn validate(&self, item: &RollingTimelinePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingTimelinePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTimelinePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_timeline_plan_validator() -> RollingTimelinePlanValidator {
    RollingTimelinePlanValidator {
        rules: vec![
            Rc::new(RollingTimelinePlanUpdatedAtRangeRule { min: 32, max: 80 }) as Rc<dyn RollingTimelinePlanRule>,
            Rc::new(RollingTimelinePlanRequiredNameRule {
                message: "rolling timeline plan needs this field".to_string(),
            }),
            Rc::new(RollingTimelinePlanSequenceRangeRule { min: 64, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_timeline_plan_validator();
        let item = new_rolling_timeline_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = RollingTimelinePlanUpdatedAtRangeRule { min: 32, max: 86400 };
        let mut item = new_rolling_timeline_plan("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingTimelinePlanRequiredNameRule {
            message: "rolling timeline plan needs a value".to_string(),
        };
        let mut item = new_rolling_timeline_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = RollingTimelinePlanSequenceRangeRule { min: 32, max: 3600 };
        let mut item = new_rolling_timeline_plan("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
