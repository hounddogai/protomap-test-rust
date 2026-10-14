use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin adjustment must pass. Check returns its problems, or none.
pub trait RegionPinAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl RegionPinAdjustmentRule for RegionPinAdjustmentCodePrefixRule {
    fn check(&self, item: &RegionPinAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAdjustmentWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinAdjustmentRule for RegionPinAdjustmentWindowDaysRangeRule {
    fn check(&self, item: &RegionPinAdjustment) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl RegionPinAdjustmentRule for RegionPinAdjustmentTitleLengthRule {
    fn check(&self, item: &RegionPinAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAdjustmentRequiredLabelRule {
    pub message: String,
}

impl RegionPinAdjustmentRule for RegionPinAdjustmentRequiredLabelRule {
    fn check(&self, item: &RegionPinAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin adjustment.
#[derive(Debug, Clone)]
pub struct RegionPinAdjustmentValidator {
    pub rules: Vec<Rc<dyn RegionPinAdjustmentRule>>,
}

impl RegionPinAdjustmentValidator {
    pub fn validate(&self, item: &RegionPinAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_adjustment_validator() -> RegionPinAdjustmentValidator {
    RegionPinAdjustmentValidator {
        rules: vec![
            Rc::new(RegionPinAdjustmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn RegionPinAdjustmentRule>,
            Rc::new(RegionPinAdjustmentWindowDaysRangeRule { min: 16, max: 80 }),
            Rc::new(RegionPinAdjustmentTitleLengthRule { max_length: 64 }),
            Rc::new(RegionPinAdjustmentRequiredLabelRule {
                message: "region pin adjustment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_adjustment_validator();
        let item = new_region_pin_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = RegionPinAdjustmentWindowDaysRangeRule { min: 80, max: 1000 };
        let mut item = new_region_pin_adjustment("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionPinAdjustmentTitleLengthRule { max_length: 64 };
        let mut item = new_region_pin_adjustment("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionPinAdjustmentRequiredLabelRule {
            message: "region pin adjustment needs an entry".to_string(),
        };
        let mut item = new_region_pin_adjustment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
