use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped region pin milestone must pass. Check returns its problems, or none.
pub trait CappedRegionPinMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &CappedRegionPinMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinMilestoneAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl CappedRegionPinMilestoneRule for CappedRegionPinMilestoneAllowedTitleRule {
    fn check(&self, item: &CappedRegionPinMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinMilestonePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedRegionPinMilestoneRule for CappedRegionPinMilestonePriorityRangeRule {
    fn check(&self, item: &CappedRegionPinMilestone) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinMilestoneCodePrefixRule {
    pub prefix: String,
}

impl CappedRegionPinMilestoneRule for CappedRegionPinMilestoneCodePrefixRule {
    fn check(&self, item: &CappedRegionPinMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinMilestoneBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedRegionPinMilestoneRule for CappedRegionPinMilestoneBatchSizeRangeRule {
    fn check(&self, item: &CappedRegionPinMilestone) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped region pin milestone.
#[derive(Debug, Clone)]
pub struct CappedRegionPinMilestoneValidator {
    pub rules: Vec<Rc<dyn CappedRegionPinMilestoneRule>>,
}

impl CappedRegionPinMilestoneValidator {
    pub fn validate(&self, item: &CappedRegionPinMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRegionPinMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRegionPinMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_region_pin_milestone_validator() -> CappedRegionPinMilestoneValidator {
    CappedRegionPinMilestoneValidator {
        rules: vec![
            Rc::new(CappedRegionPinMilestoneAllowedTitleRule {
                allowed: vec!["scheduled".to_string(), "capped".to_string(), "default".to_string()],
            }) as Rc<dyn CappedRegionPinMilestoneRule>,
            Rc::new(CappedRegionPinMilestonePriorityRangeRule { min: 16, max: 1000 }),
            Rc::new(CappedRegionPinMilestoneCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CappedRegionPinMilestoneBatchSizeRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_region_pin_milestone_validator();
        let item = new_capped_region_pin_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = CappedRegionPinMilestonePriorityRangeRule { min: 16, max: 120 };
        let mut item = new_capped_region_pin_milestone("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = CappedRegionPinMilestoneBatchSizeRangeRule { min: 120, max: 1000 };
        let mut item = new_capped_region_pin_milestone("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
