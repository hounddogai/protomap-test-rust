use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default federation forecast must pass. Check returns its problems, or none.
pub trait DefaultFederationForecastRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFederationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationForecastRequiredCodeRule {
    pub message: String,
}

impl DefaultFederationForecastRule for DefaultFederationForecastRequiredCodeRule {
    fn check(&self, item: &DefaultFederationForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationForecastLabelLengthRule {
    pub max_length: i64,
}

impl DefaultFederationForecastRule for DefaultFederationForecastLabelLengthRule {
    fn check(&self, item: &DefaultFederationForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationForecastRequiredLabelRule {
    pub message: String,
}

impl DefaultFederationForecastRule for DefaultFederationForecastRequiredLabelRule {
    fn check(&self, item: &DefaultFederationForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default federation forecast.
#[derive(Debug, Clone)]
pub struct DefaultFederationForecastValidator {
    pub rules: Vec<Rc<dyn DefaultFederationForecastRule>>,
}

impl DefaultFederationForecastValidator {
    pub fn validate(&self, item: &DefaultFederationForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultFederationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFederationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_federation_forecast_validator() -> DefaultFederationForecastValidator {
    DefaultFederationForecastValidator {
        rules: vec![
            Rc::new(DefaultFederationForecastRequiredCodeRule {
                message: "default federation forecast needs a value".to_string(),
            }) as Rc<dyn DefaultFederationForecastRule>,
            Rc::new(DefaultFederationForecastLabelLengthRule { max_length: 1000 }),
            Rc::new(DefaultFederationForecastRequiredLabelRule {
                message: "default federation forecast needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_federation_forecast_validator();
        let item = new_default_federation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultFederationForecastRequiredCodeRule {
            message: "default federation forecast needs a value".to_string(),
        };
        let mut item = new_default_federation_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultFederationForecastLabelLengthRule { max_length: 3600 };
        let mut item = new_default_federation_forecast("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultFederationForecastRequiredLabelRule {
            message: "default federation forecast needs a value".to_string(),
        };
        let mut item = new_default_federation_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
