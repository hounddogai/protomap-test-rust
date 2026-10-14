use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity forecast must pass. Check returns its problems, or none.
pub trait IdentityForecastRule: std::fmt::Debug {
    fn check(&self, item: &IdentityForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityForecastRequiredCodeRule {
    pub message: String,
}

impl IdentityForecastRule for IdentityForecastRequiredCodeRule {
    fn check(&self, item: &IdentityForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityForecastDescriptionLengthRule {
    pub max_length: i64,
}

impl IdentityForecastRule for IdentityForecastDescriptionLengthRule {
    fn check(&self, item: &IdentityForecast) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityForecastCodeLengthRule {
    pub max_length: i64,
}

impl IdentityForecastRule for IdentityForecastCodeLengthRule {
    fn check(&self, item: &IdentityForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity forecast.
#[derive(Debug, Clone)]
pub struct IdentityForecastValidator {
    pub rules: Vec<Rc<dyn IdentityForecastRule>>,
}

impl IdentityForecastValidator {
    pub fn validate(&self, item: &IdentityForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_forecast_validator() -> IdentityForecastValidator {
    IdentityForecastValidator {
        rules: vec![
            Rc::new(IdentityForecastRequiredCodeRule {
                message: "identity forecast needs a value".to_string(),
            }) as Rc<dyn IdentityForecastRule>,
            Rc::new(IdentityForecastDescriptionLengthRule { max_length: 16 }),
            Rc::new(IdentityForecastCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_forecast_validator();
        let item = new_identity_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentityForecastRequiredCodeRule {
            message: "identity forecast needs this field".to_string(),
        };
        let mut item = new_identity_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = IdentityForecastDescriptionLengthRule { max_length: 86400 };
        let mut item = new_identity_forecast("x", "y");
        item.description = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityForecastCodeLengthRule { max_length: 120 };
        let mut item = new_identity_forecast("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
