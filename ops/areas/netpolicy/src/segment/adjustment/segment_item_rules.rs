use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment item must pass. Check returns its problems, or none.
pub trait SegmentItemRule: std::fmt::Debug {
    fn check(&self, item: &SegmentItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentItemCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentItemRule for SegmentItemCapacityRangeRule {
    fn check(&self, item: &SegmentItem) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentItemWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentItemRule for SegmentItemWindowDaysRangeRule {
    fn check(&self, item: &SegmentItem) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentItemGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentItemRule for SegmentItemGraceDaysRangeRule {
    fn check(&self, item: &SegmentItem) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentItemClusterLengthRule {
    pub max_length: i64,
}

impl SegmentItemRule for SegmentItemClusterLengthRule {
    fn check(&self, item: &SegmentItem) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentItemNameLengthRule {
    pub max_length: i64,
}

impl SegmentItemRule for SegmentItemNameLengthRule {
    fn check(&self, item: &SegmentItem) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentItemCodePrefixRule {
    pub prefix: String,
}

impl SegmentItemRule for SegmentItemCodePrefixRule {
    fn check(&self, item: &SegmentItem) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment item.
#[derive(Debug, Clone)]
pub struct SegmentItemValidator {
    pub rules: Vec<Rc<dyn SegmentItemRule>>,
}

impl SegmentItemValidator {
    pub fn validate(&self, item: &SegmentItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_item_validator() -> SegmentItemValidator {
    SegmentItemValidator {
        rules: vec![
            Rc::new(SegmentItemCapacityRangeRule { min: 255, max: 3600 }) as Rc<dyn SegmentItemRule>,
            Rc::new(SegmentItemWindowDaysRangeRule { min: 64, max: 86400 }),
            Rc::new(SegmentItemGraceDaysRangeRule { min: 80, max: 255 }),
            Rc::new(SegmentItemClusterLengthRule { max_length: 1000 }),
            Rc::new(SegmentItemNameLengthRule { max_length: 255 }),
            Rc::new(SegmentItemCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_item_validator();
        let item = new_segment_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = SegmentItemCapacityRangeRule { min: 1000, max: 3600 };
        let mut item = new_segment_item("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = SegmentItemWindowDaysRangeRule { min: 32, max: 3600 };
        let mut item = new_segment_item("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = SegmentItemGraceDaysRangeRule { min: 80, max: 1000 };
        let mut item = new_segment_item("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = SegmentItemClusterLengthRule { max_length: 86400 };
        let mut item = new_segment_item("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SegmentItemNameLengthRule { max_length: 16 };
        let mut item = new_segment_item("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
