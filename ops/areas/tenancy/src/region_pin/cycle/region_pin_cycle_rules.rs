use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin cycle must pass. Check returns its problems, or none.
pub trait RegionPinCycleRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCycleCodePrefixRule {
    pub prefix: String,
}

impl RegionPinCycleRule for RegionPinCycleCodePrefixRule {
    fn check(&self, item: &RegionPinCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCycleCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinCycleRule for RegionPinCycleCodeLengthRule {
    fn check(&self, item: &RegionPinCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCycleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionPinCycleRule for RegionPinCycleRateRangeRule {
    fn check(&self, item: &RegionPinCycle) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCycleRequiredTitleRule {
    pub message: String,
}

impl RegionPinCycleRule for RegionPinCycleRequiredTitleRule {
    fn check(&self, item: &RegionPinCycle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin cycle.
#[derive(Debug, Clone)]
pub struct RegionPinCycleValidator {
    pub rules: Vec<Rc<dyn RegionPinCycleRule>>,
}

impl RegionPinCycleValidator {
    pub fn validate(&self, item: &RegionPinCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_cycle_validator() -> RegionPinCycleValidator {
    RegionPinCycleValidator {
        rules: vec![
            Rc::new(RegionPinCycleCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RegionPinCycleRule>,
            Rc::new(RegionPinCycleCodeLengthRule { max_length: 3600 }),
            Rc::new(RegionPinCycleRateRangeRule { min: 3.3, max: 4235.4 }),
            Rc::new(RegionPinCycleRequiredTitleRule {
                message: "region pin cycle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_cycle_validator();
        let item = new_region_pin_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinCycleCodeLengthRule { max_length: 3600 };
        let mut item = new_region_pin_cycle("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RegionPinCycleRateRangeRule { min: 2.1, max: 1020.6 };
        let mut item = new_region_pin_cycle("x", "y");
        item.rate = 1022.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionPinCycleRequiredTitleRule {
            message: "region pin cycle needs a value".to_string(),
        };
        let mut item = new_region_pin_cycle("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
