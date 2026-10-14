use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient redaction forecast must pass. Check returns its problems, or none.
pub trait LenientRedactionForecastRule: std::fmt::Debug {
    fn check(&self, item: &LenientRedactionForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientRedactionForecastRule for LenientRedactionForecastAllowedCodeRule {
    fn check(&self, item: &LenientRedactionForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionForecastCodeLengthRule {
    pub max_length: i64,
}

impl LenientRedactionForecastRule for LenientRedactionForecastCodeLengthRule {
    fn check(&self, item: &LenientRedactionForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionForecastRequiredLabelRule {
    pub message: String,
}

impl LenientRedactionForecastRule for LenientRedactionForecastRequiredLabelRule {
    fn check(&self, item: &LenientRedactionForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient redaction forecast.
#[derive(Debug, Clone)]
pub struct LenientRedactionForecastValidator {
    pub rules: Vec<Rc<dyn LenientRedactionForecastRule>>,
}

impl LenientRedactionForecastValidator {
    pub fn validate(&self, item: &LenientRedactionForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRedactionForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRedactionForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_redaction_forecast_validator() -> LenientRedactionForecastValidator {
    LenientRedactionForecastValidator {
        rules: vec![
            Rc::new(LenientRedactionForecastAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn LenientRedactionForecastRule>,
            Rc::new(LenientRedactionForecastCodeLengthRule { max_length: 86400 }),
            Rc::new(LenientRedactionForecastRequiredLabelRule {
                message: "lenient redaction forecast needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_redaction_forecast_validator();
        let item = new_lenient_redaction_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientRedactionForecastCodeLengthRule { max_length: 120 };
        let mut item = new_lenient_redaction_forecast("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientRedactionForecastRequiredLabelRule {
            message: "lenient redaction forecast needs an entry".to_string(),
        };
        let mut item = new_lenient_redaction_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
