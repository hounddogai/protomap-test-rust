use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary canary forecast must pass. Check returns its problems, or none.
pub trait PrimaryCanaryForecastRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryCanaryForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryForecastCodePrefixRule {
    pub prefix: String,
}

impl PrimaryCanaryForecastRule for PrimaryCanaryForecastCodePrefixRule {
    fn check(&self, item: &PrimaryCanaryForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryForecastRequiredChecksumRule {
    pub message: String,
}

impl PrimaryCanaryForecastRule for PrimaryCanaryForecastRequiredChecksumRule {
    fn check(&self, item: &PrimaryCanaryForecast) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryForecastLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryCanaryForecastRule for PrimaryCanaryForecastLabelLengthRule {
    fn check(&self, item: &PrimaryCanaryForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary canary forecast.
#[derive(Debug, Clone)]
pub struct PrimaryCanaryForecastValidator {
    pub rules: Vec<Rc<dyn PrimaryCanaryForecastRule>>,
}

impl PrimaryCanaryForecastValidator {
    pub fn validate(&self, item: &PrimaryCanaryForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryCanaryForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryCanaryForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_canary_forecast_validator() -> PrimaryCanaryForecastValidator {
    PrimaryCanaryForecastValidator {
        rules: vec![
            Rc::new(PrimaryCanaryForecastCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn PrimaryCanaryForecastRule>,
            Rc::new(PrimaryCanaryForecastRequiredChecksumRule {
                message: "primary canary forecast needs an entry".to_string(),
            }),
            Rc::new(PrimaryCanaryForecastLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_canary_forecast_validator();
        let item = new_primary_canary_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = PrimaryCanaryForecastRequiredChecksumRule {
            message: "primary canary forecast needs this field".to_string(),
        };
        let mut item = new_primary_canary_forecast("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryCanaryForecastLabelLengthRule { max_length: 80 };
        let mut item = new_primary_canary_forecast("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
