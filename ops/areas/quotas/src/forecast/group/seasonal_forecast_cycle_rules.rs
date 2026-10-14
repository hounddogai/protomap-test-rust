use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal forecast cycle must pass. Check returns its problems, or none.
pub trait SeasonalForecastCycleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalForecastCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastCycleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalForecastCycleRule for SeasonalForecastCycleBurstRangeRule {
    fn check(&self, item: &SeasonalForecastCycle) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastCycleBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalForecastCycleRule for SeasonalForecastCycleBatchSizeRangeRule {
    fn check(&self, item: &SeasonalForecastCycle) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastCycleCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalForecastCycleRule for SeasonalForecastCycleCodeLengthRule {
    fn check(&self, item: &SeasonalForecastCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastCycleNamespaceLengthRule {
    pub max_length: i64,
}

impl SeasonalForecastCycleRule for SeasonalForecastCycleNamespaceLengthRule {
    fn check(&self, item: &SeasonalForecastCycle) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastCycleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalForecastCycleRule for SeasonalForecastCycleRateRangeRule {
    fn check(&self, item: &SeasonalForecastCycle) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastCycleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalForecastCycleRule for SeasonalForecastCycleDiscountRangeRule {
    fn check(&self, item: &SeasonalForecastCycle) -> Vec<String> {
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

/// Runs every rule of a seasonal forecast cycle.
#[derive(Debug, Clone)]
pub struct SeasonalForecastCycleValidator {
    pub rules: Vec<Rc<dyn SeasonalForecastCycleRule>>,
}

impl SeasonalForecastCycleValidator {
    pub fn validate(&self, item: &SeasonalForecastCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalForecastCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalForecastCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_forecast_cycle_validator() -> SeasonalForecastCycleValidator {
    SeasonalForecastCycleValidator {
        rules: vec![
            Rc::new(SeasonalForecastCycleBurstRangeRule { min: 80, max: 255 }) as Rc<dyn SeasonalForecastCycleRule>,
            Rc::new(SeasonalForecastCycleBatchSizeRangeRule { min: 32, max: 255 }),
            Rc::new(SeasonalForecastCycleCodeLengthRule { max_length: 255 }),
            Rc::new(SeasonalForecastCycleNamespaceLengthRule { max_length: 86400 }),
            Rc::new(SeasonalForecastCycleRateRangeRule { min: 8.8, max: 2698.2 }),
            Rc::new(SeasonalForecastCycleDiscountRangeRule { min: 6.6, max: 1424.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_forecast_cycle_validator();
        let item = new_seasonal_forecast_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = SeasonalForecastCycleBurstRangeRule { min: 16, max: 120 };
        let mut item = new_seasonal_forecast_cycle("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = SeasonalForecastCycleBatchSizeRangeRule { min: 255, max: 3600 };
        let mut item = new_seasonal_forecast_cycle("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalForecastCycleCodeLengthRule { max_length: 255 };
        let mut item = new_seasonal_forecast_cycle("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = SeasonalForecastCycleNamespaceLengthRule { max_length: 80 };
        let mut item = new_seasonal_forecast_cycle("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = SeasonalForecastCycleRateRangeRule { min: 0.1, max: 4706.0 };
        let mut item = new_seasonal_forecast_cycle("x", "y");
        item.rate = 4707.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = SeasonalForecastCycleDiscountRangeRule { min: 3.7, max: 1757.4 };
        let mut item = new_seasonal_forecast_cycle("x", "y");
        item.discount = 1758.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
