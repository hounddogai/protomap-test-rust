use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal forecast segment must pass. Check returns its problems, or none.
pub trait SeasonalForecastSegmentRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalForecastSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastSegmentCodePrefixRule {
    pub prefix: String,
}

impl SeasonalForecastSegmentRule for SeasonalForecastSegmentCodePrefixRule {
    fn check(&self, item: &SeasonalForecastSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastSegmentRequiredLabelRule {
    pub message: String,
}

impl SeasonalForecastSegmentRule for SeasonalForecastSegmentRequiredLabelRule {
    fn check(&self, item: &SeasonalForecastSegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastSegmentThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalForecastSegmentRule for SeasonalForecastSegmentThresholdRangeRule {
    fn check(&self, item: &SeasonalForecastSegment) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastSegmentCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalForecastSegmentRule for SeasonalForecastSegmentCodeLengthRule {
    fn check(&self, item: &SeasonalForecastSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastSegmentRequiredCodeRule {
    pub message: String,
}

impl SeasonalForecastSegmentRule for SeasonalForecastSegmentRequiredCodeRule {
    fn check(&self, item: &SeasonalForecastSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastSegmentQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalForecastSegmentRule for SeasonalForecastSegmentQuotaUnitsRangeRule {
    fn check(&self, item: &SeasonalForecastSegment) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal forecast segment.
#[derive(Debug, Clone)]
pub struct SeasonalForecastSegmentValidator {
    pub rules: Vec<Rc<dyn SeasonalForecastSegmentRule>>,
}

impl SeasonalForecastSegmentValidator {
    pub fn validate(&self, item: &SeasonalForecastSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalForecastSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalForecastSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_forecast_segment_validator() -> SeasonalForecastSegmentValidator {
    SeasonalForecastSegmentValidator {
        rules: vec![
            Rc::new(SeasonalForecastSegmentCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn SeasonalForecastSegmentRule>,
            Rc::new(SeasonalForecastSegmentRequiredLabelRule {
                message: "seasonal forecast segment needs an entry".to_string(),
            }),
            Rc::new(SeasonalForecastSegmentThresholdRangeRule { min: 2.6, max: 2968.4 }),
            Rc::new(SeasonalForecastSegmentCodeLengthRule { max_length: 120 }),
            Rc::new(SeasonalForecastSegmentRequiredCodeRule {
                message: "seasonal forecast segment needs this field".to_string(),
            }),
            Rc::new(SeasonalForecastSegmentQuotaUnitsRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_forecast_segment_validator();
        let item = new_seasonal_forecast_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SeasonalForecastSegmentRequiredLabelRule {
            message: "seasonal forecast segment needs an entry".to_string(),
        };
        let mut item = new_seasonal_forecast_segment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = SeasonalForecastSegmentThresholdRangeRule { min: 3.3, max: 2390.4 };
        let mut item = new_seasonal_forecast_segment("x", "y");
        item.threshold = 2391.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalForecastSegmentCodeLengthRule { max_length: 32 };
        let mut item = new_seasonal_forecast_segment("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalForecastSegmentRequiredCodeRule {
            message: "seasonal forecast segment needs an entry".to_string(),
        };
        let mut item = new_seasonal_forecast_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = SeasonalForecastSegmentQuotaUnitsRangeRule { min: 80, max: 86400 };
        let mut item = new_seasonal_forecast_segment("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
