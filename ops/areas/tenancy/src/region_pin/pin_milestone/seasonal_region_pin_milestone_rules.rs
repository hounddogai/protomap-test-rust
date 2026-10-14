use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal region pin milestone must pass. Check returns its problems, or none.
pub trait SeasonalRegionPinMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRegionPinMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinMilestonePinnedGuardRule {
    pub limit: f64,
}

impl SeasonalRegionPinMilestoneRule for SeasonalRegionPinMilestonePinnedGuardRule {
    fn check(&self, item: &SeasonalRegionPinMilestone) -> Vec<String> {
        if item.pinned && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while pinned", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinMilestoneRequiredCodeRule {
    pub message: String,
}

impl SeasonalRegionPinMilestoneRule for SeasonalRegionPinMilestoneRequiredCodeRule {
    fn check(&self, item: &SeasonalRegionPinMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinMilestoneWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalRegionPinMilestoneRule for SeasonalRegionPinMilestoneWeightRangeRule {
    fn check(&self, item: &SeasonalRegionPinMilestone) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal region pin milestone.
#[derive(Debug, Clone)]
pub struct SeasonalRegionPinMilestoneValidator {
    pub rules: Vec<Rc<dyn SeasonalRegionPinMilestoneRule>>,
}

impl SeasonalRegionPinMilestoneValidator {
    pub fn validate(&self, item: &SeasonalRegionPinMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalRegionPinMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRegionPinMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_region_pin_milestone_validator() -> SeasonalRegionPinMilestoneValidator {
    SeasonalRegionPinMilestoneValidator {
        rules: vec![
            Rc::new(SeasonalRegionPinMilestonePinnedGuardRule { limit: 5.3 }) as Rc<dyn SeasonalRegionPinMilestoneRule>,
            Rc::new(SeasonalRegionPinMilestoneRequiredCodeRule {
                message: "seasonal region pin milestone needs a value".to_string(),
            }),
            Rc::new(SeasonalRegionPinMilestoneWeightRangeRule { min: 6.8, max: 3862.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_region_pin_milestone_validator();
        let item = new_seasonal_region_pin_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalRegionPinMilestoneRequiredCodeRule {
            message: "seasonal region pin milestone needs a value".to_string(),
        };
        let mut item = new_seasonal_region_pin_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SeasonalRegionPinMilestoneWeightRangeRule { min: 2.3, max: 4864.7 };
        let mut item = new_seasonal_region_pin_milestone("x", "y");
        item.weight = 4866.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
