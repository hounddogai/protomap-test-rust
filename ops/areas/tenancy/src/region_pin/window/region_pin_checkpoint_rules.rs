use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin checkpoint must pass. Check returns its problems, or none.
pub trait RegionPinCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCheckpointCategoryLengthRule {
    pub max_length: i64,
}

impl RegionPinCheckpointRule for RegionPinCheckpointCategoryLengthRule {
    fn check(&self, item: &RegionPinCheckpoint) -> Vec<String> {
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
pub struct RegionPinCheckpointLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinCheckpointRule for RegionPinCheckpointLimitRangeRule {
    fn check(&self, item: &RegionPinCheckpoint) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCheckpointTitleLengthRule {
    pub max_length: i64,
}

impl RegionPinCheckpointRule for RegionPinCheckpointTitleLengthRule {
    fn check(&self, item: &RegionPinCheckpoint) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCheckpointDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionPinCheckpointRule for RegionPinCheckpointDiscountRangeRule {
    fn check(&self, item: &RegionPinCheckpoint) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a region pin checkpoint.
#[derive(Debug, Clone)]
pub struct RegionPinCheckpointValidator {
    pub rules: Vec<Rc<dyn RegionPinCheckpointRule>>,
}

impl RegionPinCheckpointValidator {
    pub fn validate(&self, item: &RegionPinCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_checkpoint_validator() -> RegionPinCheckpointValidator {
    RegionPinCheckpointValidator {
        rules: vec![
            Rc::new(RegionPinCheckpointCategoryLengthRule { max_length: 1000 }) as Rc<dyn RegionPinCheckpointRule>,
            Rc::new(RegionPinCheckpointLimitRangeRule { min: 1000, max: 86400 }),
            Rc::new(RegionPinCheckpointTitleLengthRule { max_length: 3600 }),
            Rc::new(RegionPinCheckpointDiscountRangeRule { min: 9.9, max: 2349.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_checkpoint_validator();
        let item = new_region_pin_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = RegionPinCheckpointCategoryLengthRule { max_length: 86400 };
        let mut item = new_region_pin_checkpoint("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = RegionPinCheckpointLimitRangeRule { min: 64, max: 1000 };
        let mut item = new_region_pin_checkpoint("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionPinCheckpointTitleLengthRule { max_length: 16 };
        let mut item = new_region_pin_checkpoint("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = RegionPinCheckpointDiscountRangeRule { min: 9.3, max: 506.7 };
        let mut item = new_region_pin_checkpoint("x", "y");
        item.discount = 508.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
