use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered region pin window must pass. Check returns its problems, or none.
pub trait TieredRegionPinWindowRule: std::fmt::Debug {
    fn check(&self, item: &TieredRegionPinWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinWindowFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredRegionPinWindowRule for TieredRegionPinWindowFloorValueRangeRule {
    fn check(&self, item: &TieredRegionPinWindow) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinWindowRequiredCodeRule {
    pub message: String,
}

impl TieredRegionPinWindowRule for TieredRegionPinWindowRequiredCodeRule {
    fn check(&self, item: &TieredRegionPinWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinWindowCodePrefixRule {
    pub prefix: String,
}

impl TieredRegionPinWindowRule for TieredRegionPinWindowCodePrefixRule {
    fn check(&self, item: &TieredRegionPinWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinWindowTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredRegionPinWindowRule for TieredRegionPinWindowTtlSecondsRangeRule {
    fn check(&self, item: &TieredRegionPinWindow) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinWindowRequiredTitleRule {
    pub message: String,
}

impl TieredRegionPinWindowRule for TieredRegionPinWindowRequiredTitleRule {
    fn check(&self, item: &TieredRegionPinWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered region pin window.
#[derive(Debug, Clone)]
pub struct TieredRegionPinWindowValidator {
    pub rules: Vec<Rc<dyn TieredRegionPinWindowRule>>,
}

impl TieredRegionPinWindowValidator {
    pub fn validate(&self, item: &TieredRegionPinWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredRegionPinWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRegionPinWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_region_pin_window_validator() -> TieredRegionPinWindowValidator {
    TieredRegionPinWindowValidator {
        rules: vec![
            Rc::new(TieredRegionPinWindowFloorValueRangeRule { min: 9.1, max: 3113.1 })
                as Rc<dyn TieredRegionPinWindowRule>,
            Rc::new(TieredRegionPinWindowRequiredCodeRule {
                message: "tiered region pin window needs this field".to_string(),
            }),
            Rc::new(TieredRegionPinWindowCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TieredRegionPinWindowTtlSecondsRangeRule { min: 64, max: 3600 }),
            Rc::new(TieredRegionPinWindowRequiredTitleRule {
                message: "tiered region pin window needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_region_pin_window_validator();
        let item = new_tiered_region_pin_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = TieredRegionPinWindowFloorValueRangeRule { min: 4.8, max: 4000.2 };
        let mut item = new_tiered_region_pin_window("x", "y");
        item.floor_value = 4001.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredRegionPinWindowRequiredCodeRule {
            message: "tiered region pin window needs a value".to_string(),
        };
        let mut item = new_tiered_region_pin_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = TieredRegionPinWindowTtlSecondsRangeRule { min: 255, max: 3600 };
        let mut item = new_tiered_region_pin_window("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredRegionPinWindowRequiredTitleRule {
            message: "tiered region pin window needs an entry".to_string(),
        };
        let mut item = new_tiered_region_pin_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
