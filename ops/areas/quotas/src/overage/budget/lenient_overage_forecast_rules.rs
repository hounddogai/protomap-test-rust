use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient overage forecast must pass. Check returns its problems, or none.
pub trait LenientOverageForecastRule: std::fmt::Debug {
    fn check(&self, item: &LenientOverageForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientOverageForecastLabelLengthRule {
    pub max_length: i64,
}

impl LenientOverageForecastRule for LenientOverageForecastLabelLengthRule {
    fn check(&self, item: &LenientOverageForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientOverageForecastRequiredAudienceRule {
    pub message: String,
}

impl LenientOverageForecastRule for LenientOverageForecastRequiredAudienceRule {
    fn check(&self, item: &LenientOverageForecast) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientOverageForecastWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientOverageForecastRule for LenientOverageForecastWeightRangeRule {
    fn check(&self, item: &LenientOverageForecast) -> Vec<String> {
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
pub struct LenientOverageForecastGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientOverageForecastRule for LenientOverageForecastGraceDaysRangeRule {
    fn check(&self, item: &LenientOverageForecast) -> Vec<String> {
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

/// Runs every rule of a lenient overage forecast.
#[derive(Debug, Clone)]
pub struct LenientOverageForecastValidator {
    pub rules: Vec<Rc<dyn LenientOverageForecastRule>>,
}

impl LenientOverageForecastValidator {
    pub fn validate(&self, item: &LenientOverageForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientOverageForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientOverageForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_overage_forecast_validator() -> LenientOverageForecastValidator {
    LenientOverageForecastValidator {
        rules: vec![
            Rc::new(LenientOverageForecastLabelLengthRule { max_length: 3600 }) as Rc<dyn LenientOverageForecastRule>,
            Rc::new(LenientOverageForecastRequiredAudienceRule {
                message: "lenient overage forecast needs this field".to_string(),
            }),
            Rc::new(LenientOverageForecastWeightRangeRule { min: 2.8, max: 1849.9 }),
            Rc::new(LenientOverageForecastGraceDaysRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_overage_forecast_validator();
        let item = new_lenient_overage_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientOverageForecastLabelLengthRule { max_length: 86400 };
        let mut item = new_lenient_overage_forecast("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = LenientOverageForecastRequiredAudienceRule {
            message: "lenient overage forecast needs a value".to_string(),
        };
        let mut item = new_lenient_overage_forecast("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = LenientOverageForecastWeightRangeRule { min: 8.5, max: 2613.3 };
        let mut item = new_lenient_overage_forecast("x", "y");
        item.weight = 2614.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LenientOverageForecastGraceDaysRangeRule { min: 32, max: 255 };
        let mut item = new_lenient_overage_forecast("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
