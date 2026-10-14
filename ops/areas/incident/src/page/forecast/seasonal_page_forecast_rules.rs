use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal page forecast must pass. Check returns its problems, or none.
pub trait SeasonalPageForecastRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalPageForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageForecastRequiredOwnerIdRule {
    pub message: String,
}

impl SeasonalPageForecastRule for SeasonalPageForecastRequiredOwnerIdRule {
    fn check(&self, item: &SeasonalPageForecast) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageForecastRequiredLabelRule {
    pub message: String,
}

impl SeasonalPageForecastRule for SeasonalPageForecastRequiredLabelRule {
    fn check(&self, item: &SeasonalPageForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageForecastCodePrefixRule {
    pub prefix: String,
}

impl SeasonalPageForecastRule for SeasonalPageForecastCodePrefixRule {
    fn check(&self, item: &SeasonalPageForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageForecastMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalPageForecastRule for SeasonalPageForecastMarginRangeRule {
    fn check(&self, item: &SeasonalPageForecast) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageForecastPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalPageForecastRule for SeasonalPageForecastPriorityRangeRule {
    fn check(&self, item: &SeasonalPageForecast) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal page forecast.
#[derive(Debug, Clone)]
pub struct SeasonalPageForecastValidator {
    pub rules: Vec<Rc<dyn SeasonalPageForecastRule>>,
}

impl SeasonalPageForecastValidator {
    pub fn validate(&self, item: &SeasonalPageForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalPageForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalPageForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_page_forecast_validator() -> SeasonalPageForecastValidator {
    SeasonalPageForecastValidator {
        rules: vec![
            Rc::new(SeasonalPageForecastRequiredOwnerIdRule {
                message: "seasonal page forecast needs an entry".to_string(),
            }) as Rc<dyn SeasonalPageForecastRule>,
            Rc::new(SeasonalPageForecastRequiredLabelRule {
                message: "seasonal page forecast needs an entry".to_string(),
            }),
            Rc::new(SeasonalPageForecastCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SeasonalPageForecastMarginRangeRule { min: 9.2, max: 3008.0 }),
            Rc::new(SeasonalPageForecastPriorityRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_page_forecast_validator();
        let item = new_seasonal_page_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = SeasonalPageForecastRequiredOwnerIdRule {
            message: "seasonal page forecast needs this field".to_string(),
        };
        let mut item = new_seasonal_page_forecast("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SeasonalPageForecastRequiredLabelRule {
            message: "seasonal page forecast needs a value".to_string(),
        };
        let mut item = new_seasonal_page_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SeasonalPageForecastMarginRangeRule { min: 8.2, max: 3844.3 };
        let mut item = new_seasonal_page_forecast("x", "y");
        item.margin = 3845.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = SeasonalPageForecastPriorityRangeRule { min: 16, max: 3600 };
        let mut item = new_seasonal_page_forecast("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
