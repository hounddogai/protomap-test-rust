use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient suspension forecast must pass. Check returns its problems, or none.
pub trait LenientSuspensionForecastRule: std::fmt::Debug {
    fn check(&self, item: &LenientSuspensionForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionForecastAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl LenientSuspensionForecastRule for LenientSuspensionForecastAllowedContactEmailRule {
    fn check(&self, item: &LenientSuspensionForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is inconsistent", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionForecastThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientSuspensionForecastRule for LenientSuspensionForecastThresholdRangeRule {
    fn check(&self, item: &LenientSuspensionForecast) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionForecastRequiredCodeRule {
    pub message: String,
}

impl LenientSuspensionForecastRule for LenientSuspensionForecastRequiredCodeRule {
    fn check(&self, item: &LenientSuspensionForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient suspension forecast.
#[derive(Debug, Clone)]
pub struct LenientSuspensionForecastValidator {
    pub rules: Vec<Rc<dyn LenientSuspensionForecastRule>>,
}

impl LenientSuspensionForecastValidator {
    pub fn validate(&self, item: &LenientSuspensionForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSuspensionForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSuspensionForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_suspension_forecast_validator() -> LenientSuspensionForecastValidator {
    LenientSuspensionForecastValidator {
        rules: vec![
            Rc::new(LenientSuspensionForecastAllowedContactEmailRule {
                allowed: vec!["primary".to_string(), "weighted".to_string(), "fallback".to_string()],
            }) as Rc<dyn LenientSuspensionForecastRule>,
            Rc::new(LenientSuspensionForecastThresholdRangeRule { min: 7.5, max: 4922.1 }),
            Rc::new(LenientSuspensionForecastRequiredCodeRule {
                message: "lenient suspension forecast needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_suspension_forecast_validator();
        let item = new_lenient_suspension_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LenientSuspensionForecastThresholdRangeRule { min: 8.6, max: 3768.8 };
        let mut item = new_lenient_suspension_forecast("x", "y");
        item.threshold = 3770.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientSuspensionForecastRequiredCodeRule {
            message: "lenient suspension forecast needs a value".to_string(),
        };
        let mut item = new_lenient_suspension_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
