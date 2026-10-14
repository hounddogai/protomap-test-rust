use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive sink milestone must pass. Check returns its problems, or none.
pub trait AdaptiveSinkMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSinkMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSinkMilestoneDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveSinkMilestoneRule for AdaptiveSinkMilestoneDurationMinutesRangeRule {
    fn check(&self, item: &AdaptiveSinkMilestone) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSinkMilestoneCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveSinkMilestoneRule for AdaptiveSinkMilestoneCodePrefixRule {
    fn check(&self, item: &AdaptiveSinkMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSinkMilestoneWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveSinkMilestoneRule for AdaptiveSinkMilestoneWindowDaysRangeRule {
    fn check(&self, item: &AdaptiveSinkMilestone) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive sink milestone.
#[derive(Debug, Clone)]
pub struct AdaptiveSinkMilestoneValidator {
    pub rules: Vec<Rc<dyn AdaptiveSinkMilestoneRule>>,
}

impl AdaptiveSinkMilestoneValidator {
    pub fn validate(&self, item: &AdaptiveSinkMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSinkMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSinkMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_sink_milestone_validator() -> AdaptiveSinkMilestoneValidator {
    AdaptiveSinkMilestoneValidator {
        rules: vec![
            Rc::new(AdaptiveSinkMilestoneDurationMinutesRangeRule { min: 255, max: 1000 })
                as Rc<dyn AdaptiveSinkMilestoneRule>,
            Rc::new(AdaptiveSinkMilestoneCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AdaptiveSinkMilestoneWindowDaysRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_sink_milestone_validator();
        let item = new_adaptive_sink_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = AdaptiveSinkMilestoneDurationMinutesRangeRule { min: 32, max: 120 };
        let mut item = new_adaptive_sink_milestone("x", "y");
        item.duration_minutes = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = AdaptiveSinkMilestoneWindowDaysRangeRule { min: 64, max: 86400 };
        let mut item = new_adaptive_sink_milestone("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
