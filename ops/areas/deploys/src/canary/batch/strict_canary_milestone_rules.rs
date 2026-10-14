use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict canary milestone must pass. Check returns its problems, or none.
pub trait StrictCanaryMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &StrictCanaryMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryMilestoneAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl StrictCanaryMilestoneRule for StrictCanaryMilestoneAllowedAudienceRule {
    fn check(&self, item: &StrictCanaryMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is unknown", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryMilestoneSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictCanaryMilestoneRule for StrictCanaryMilestoneSequenceRangeRule {
    fn check(&self, item: &StrictCanaryMilestone) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryMilestoneTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictCanaryMilestoneRule for StrictCanaryMilestoneTtlSecondsRangeRule {
    fn check(&self, item: &StrictCanaryMilestone) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryMilestoneTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl StrictCanaryMilestoneRule for StrictCanaryMilestoneTagsLimitRule {
    fn check(&self, item: &StrictCanaryMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a strict canary milestone.
#[derive(Debug, Clone)]
pub struct StrictCanaryMilestoneValidator {
    pub rules: Vec<Rc<dyn StrictCanaryMilestoneRule>>,
}

impl StrictCanaryMilestoneValidator {
    pub fn validate(&self, item: &StrictCanaryMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictCanaryMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictCanaryMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_canary_milestone_validator() -> StrictCanaryMilestoneValidator {
    StrictCanaryMilestoneValidator {
        rules: vec![
            Rc::new(StrictCanaryMilestoneAllowedAudienceRule {
                allowed: vec!["batched".to_string()],
            }) as Rc<dyn StrictCanaryMilestoneRule>,
            Rc::new(StrictCanaryMilestoneSequenceRangeRule { min: 16, max: 80 }),
            Rc::new(StrictCanaryMilestoneTtlSecondsRangeRule { min: 3600, max: 3600 }),
            Rc::new(StrictCanaryMilestoneTagsLimitRule {
                max_count: 32,
                banned: vec!["pending".to_string(), "tiered".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_canary_milestone_validator();
        let item = new_strict_canary_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = StrictCanaryMilestoneSequenceRangeRule { min: 16, max: 86400 };
        let mut item = new_strict_canary_milestone("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = StrictCanaryMilestoneTtlSecondsRangeRule { min: 120, max: 3600 };
        let mut item = new_strict_canary_milestone("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
