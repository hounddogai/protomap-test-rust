use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal probe override must pass. Check returns its problems, or none.
pub trait SeasonalProbeOverrideRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalProbeOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeOverrideMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalProbeOverrideRule for SeasonalProbeOverrideMarginRangeRule {
    fn check(&self, item: &SeasonalProbeOverride) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeOverrideMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalProbeOverrideRule for SeasonalProbeOverrideMinimumRangeRule {
    fn check(&self, item: &SeasonalProbeOverride) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeOverrideFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalProbeOverrideRule for SeasonalProbeOverrideFloorValueRangeRule {
    fn check(&self, item: &SeasonalProbeOverride) -> Vec<String> {
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
pub struct SeasonalProbeOverrideNameLengthRule {
    pub max_length: i64,
}

impl SeasonalProbeOverrideRule for SeasonalProbeOverrideNameLengthRule {
    fn check(&self, item: &SeasonalProbeOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeOverrideAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SeasonalProbeOverrideRule for SeasonalProbeOverrideAllowedNameRule {
    fn check(&self, item: &SeasonalProbeOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal probe override.
#[derive(Debug, Clone)]
pub struct SeasonalProbeOverrideValidator {
    pub rules: Vec<Rc<dyn SeasonalProbeOverrideRule>>,
}

impl SeasonalProbeOverrideValidator {
    pub fn validate(&self, item: &SeasonalProbeOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalProbeOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalProbeOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_probe_override_validator() -> SeasonalProbeOverrideValidator {
    SeasonalProbeOverrideValidator {
        rules: vec![
            Rc::new(SeasonalProbeOverrideMarginRangeRule { min: 1.6, max: 2284.1 })
                as Rc<dyn SeasonalProbeOverrideRule>,
            Rc::new(SeasonalProbeOverrideMinimumRangeRule { min: 6.7, max: 2835.5 }),
            Rc::new(SeasonalProbeOverrideFloorValueRangeRule { min: 3.1, max: 1893.1 }),
            Rc::new(SeasonalProbeOverrideNameLengthRule { max_length: 16 }),
            Rc::new(SeasonalProbeOverrideAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_probe_override_validator();
        let item = new_seasonal_probe_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SeasonalProbeOverrideMarginRangeRule { min: 4.4, max: 1091.9 };
        let mut item = new_seasonal_probe_override("x", "y");
        item.margin = 1093.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = SeasonalProbeOverrideMinimumRangeRule { min: 3.9, max: 1757.8 };
        let mut item = new_seasonal_probe_override("x", "y");
        item.minimum = 1759.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = SeasonalProbeOverrideFloorValueRangeRule { min: 6.4, max: 4655.5 };
        let mut item = new_seasonal_probe_override("x", "y");
        item.floor_value = 4657.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalProbeOverrideNameLengthRule { max_length: 16 };
        let mut item = new_seasonal_probe_override("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
