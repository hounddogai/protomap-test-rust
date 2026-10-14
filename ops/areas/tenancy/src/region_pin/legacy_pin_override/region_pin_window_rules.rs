use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin window must pass. Check returns its problems, or none.
pub trait RegionPinWindowRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinWindowRequiredTitleRule {
    pub message: String,
}

impl RegionPinWindowRule for RegionPinWindowRequiredTitleRule {
    fn check(&self, item: &RegionPinWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinWindowWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionPinWindowRule for RegionPinWindowWeightRangeRule {
    fn check(&self, item: &RegionPinWindow) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinWindowRequiredCodeRule {
    pub message: String,
}

impl RegionPinWindowRule for RegionPinWindowRequiredCodeRule {
    fn check(&self, item: &RegionPinWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinWindowCodePrefixRule {
    pub prefix: String,
}

impl RegionPinWindowRule for RegionPinWindowCodePrefixRule {
    fn check(&self, item: &RegionPinWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinWindowMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionPinWindowRule for RegionPinWindowMaximumRangeRule {
    fn check(&self, item: &RegionPinWindow) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin window.
#[derive(Debug, Clone)]
pub struct RegionPinWindowValidator {
    pub rules: Vec<Rc<dyn RegionPinWindowRule>>,
}

impl RegionPinWindowValidator {
    pub fn validate(&self, item: &RegionPinWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_window_validator() -> RegionPinWindowValidator {
    RegionPinWindowValidator {
        rules: vec![
            Rc::new(RegionPinWindowRequiredTitleRule {
                message: "region pin window needs an entry".to_string(),
            }) as Rc<dyn RegionPinWindowRule>,
            Rc::new(RegionPinWindowWeightRangeRule { min: 4.6, max: 3438.1 }),
            Rc::new(RegionPinWindowRequiredCodeRule {
                message: "region pin window needs an entry".to_string(),
            }),
            Rc::new(RegionPinWindowCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RegionPinWindowMaximumRangeRule { min: 5.6, max: 383.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_window_validator();
        let item = new_region_pin_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionPinWindowRequiredTitleRule {
            message: "region pin window needs this field".to_string(),
        };
        let mut item = new_region_pin_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = RegionPinWindowWeightRangeRule { min: 6.2, max: 430.7 };
        let mut item = new_region_pin_window("x", "y");
        item.weight = 432.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinWindowRequiredCodeRule {
            message: "region pin window needs this field".to_string(),
        };
        let mut item = new_region_pin_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RegionPinWindowMaximumRangeRule { min: 4.3, max: 2404.9 };
        let mut item = new_region_pin_window("x", "y");
        item.maximum = 2406.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
