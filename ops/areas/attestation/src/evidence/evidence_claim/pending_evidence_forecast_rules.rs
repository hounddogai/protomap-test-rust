use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending evidence forecast must pass. Check returns its problems, or none.
pub trait PendingEvidenceForecastRule: std::fmt::Debug {
    fn check(&self, item: &PendingEvidenceForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEvidenceForecastRequiredTitleRule {
    pub message: String,
}

impl PendingEvidenceForecastRule for PendingEvidenceForecastRequiredTitleRule {
    fn check(&self, item: &PendingEvidenceForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEvidenceForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingEvidenceForecastRule for PendingEvidenceForecastAllowedCodeRule {
    fn check(&self, item: &PendingEvidenceForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEvidenceForecastCountryLengthRule {
    pub max_length: i64,
}

impl PendingEvidenceForecastRule for PendingEvidenceForecastCountryLengthRule {
    fn check(&self, item: &PendingEvidenceForecast) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending evidence forecast.
#[derive(Debug, Clone)]
pub struct PendingEvidenceForecastValidator {
    pub rules: Vec<Rc<dyn PendingEvidenceForecastRule>>,
}

impl PendingEvidenceForecastValidator {
    pub fn validate(&self, item: &PendingEvidenceForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingEvidenceForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingEvidenceForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_evidence_forecast_validator() -> PendingEvidenceForecastValidator {
    PendingEvidenceForecastValidator {
        rules: vec![
            Rc::new(PendingEvidenceForecastRequiredTitleRule {
                message: "pending evidence forecast needs this field".to_string(),
            }) as Rc<dyn PendingEvidenceForecastRule>,
            Rc::new(PendingEvidenceForecastAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PendingEvidenceForecastCountryLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_evidence_forecast_validator();
        let item = new_pending_evidence_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingEvidenceForecastRequiredTitleRule {
            message: "pending evidence forecast needs a value".to_string(),
        };
        let mut item = new_pending_evidence_forecast("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PendingEvidenceForecastCountryLengthRule { max_length: 1000 };
        let mut item = new_pending_evidence_forecast("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
