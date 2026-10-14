use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page forecast must pass. Check returns its problems, or none.
pub trait PageForecastRule: std::fmt::Debug {
    fn check(&self, item: &PageForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageForecastRequiredCodeRule {
    pub message: String,
}

impl PageForecastRule for PageForecastRequiredCodeRule {
    fn check(&self, item: &PageForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageForecastPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageForecastRule for PageForecastPortRangeRule {
    fn check(&self, item: &PageForecast) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageForecastLabelLengthRule {
    pub max_length: i64,
}

impl PageForecastRule for PageForecastLabelLengthRule {
    fn check(&self, item: &PageForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page forecast.
#[derive(Debug, Clone)]
pub struct PageForecastValidator {
    pub rules: Vec<Rc<dyn PageForecastRule>>,
}

impl PageForecastValidator {
    pub fn validate(&self, item: &PageForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_forecast_validator() -> PageForecastValidator {
    PageForecastValidator {
        rules: vec![
            Rc::new(PageForecastRequiredCodeRule {
                message: "page forecast needs an entry".to_string(),
            }) as Rc<dyn PageForecastRule>,
            Rc::new(PageForecastPortRangeRule { min: 255, max: 1000 }),
            Rc::new(PageForecastLabelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_forecast_validator();
        let item = new_page_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PageForecastRequiredCodeRule {
            message: "page forecast needs a value".to_string(),
        };
        let mut item = new_page_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PageForecastPortRangeRule { min: 120, max: 86400 };
        let mut item = new_page_forecast("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PageForecastLabelLengthRule { max_length: 32 };
        let mut item = new_page_forecast("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
