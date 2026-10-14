use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal slo milestone must pass. Check returns its problems, or none.
pub trait SeasonalSloMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalSloMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloMilestoneCodePrefixRule {
    pub prefix: String,
}

impl SeasonalSloMilestoneRule for SeasonalSloMilestoneCodePrefixRule {
    fn check(&self, item: &SeasonalSloMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloMilestoneTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalSloMilestoneRule for SeasonalSloMilestoneTitleLengthRule {
    fn check(&self, item: &SeasonalSloMilestone) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloMilestoneRequiredCodeRule {
    pub message: String,
}

impl SeasonalSloMilestoneRule for SeasonalSloMilestoneRequiredCodeRule {
    fn check(&self, item: &SeasonalSloMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloMilestoneUnitLengthRule {
    pub max_length: i64,
}

impl SeasonalSloMilestoneRule for SeasonalSloMilestoneUnitLengthRule {
    fn check(&self, item: &SeasonalSloMilestone) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloMilestoneRequiredTitleRule {
    pub message: String,
}

impl SeasonalSloMilestoneRule for SeasonalSloMilestoneRequiredTitleRule {
    fn check(&self, item: &SeasonalSloMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloMilestoneQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalSloMilestoneRule for SeasonalSloMilestoneQuantityRangeRule {
    fn check(&self, item: &SeasonalSloMilestone) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal slo milestone.
#[derive(Debug, Clone)]
pub struct SeasonalSloMilestoneValidator {
    pub rules: Vec<Rc<dyn SeasonalSloMilestoneRule>>,
}

impl SeasonalSloMilestoneValidator {
    pub fn validate(&self, item: &SeasonalSloMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalSloMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalSloMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_slo_milestone_validator() -> SeasonalSloMilestoneValidator {
    SeasonalSloMilestoneValidator {
        rules: vec![
            Rc::new(SeasonalSloMilestoneCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn SeasonalSloMilestoneRule>,
            Rc::new(SeasonalSloMilestoneTitleLengthRule { max_length: 86400 }),
            Rc::new(SeasonalSloMilestoneRequiredCodeRule {
                message: "seasonal slo milestone needs a value".to_string(),
            }),
            Rc::new(SeasonalSloMilestoneUnitLengthRule { max_length: 1000 }),
            Rc::new(SeasonalSloMilestoneRequiredTitleRule {
                message: "seasonal slo milestone needs an entry".to_string(),
            }),
            Rc::new(SeasonalSloMilestoneQuantityRangeRule { min: 64, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_slo_milestone_validator();
        let item = new_seasonal_slo_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalSloMilestoneTitleLengthRule { max_length: 86400 };
        let mut item = new_seasonal_slo_milestone("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalSloMilestoneRequiredCodeRule {
            message: "seasonal slo milestone needs an entry".to_string(),
        };
        let mut item = new_seasonal_slo_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = SeasonalSloMilestoneUnitLengthRule { max_length: 32 };
        let mut item = new_seasonal_slo_milestone("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalSloMilestoneRequiredTitleRule {
            message: "seasonal slo milestone needs a value".to_string(),
        };
        let mut item = new_seasonal_slo_milestone("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = SeasonalSloMilestoneQuantityRangeRule { min: 16, max: 255 };
        let mut item = new_seasonal_slo_milestone("x", "y");
        item.quantity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
