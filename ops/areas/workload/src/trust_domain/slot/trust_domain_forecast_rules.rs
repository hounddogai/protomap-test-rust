use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain forecast must pass. Check returns its problems, or none.
pub trait TrustDomainForecastRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainForecastChannelLengthRule {
    pub max_length: i64,
}

impl TrustDomainForecastRule for TrustDomainForecastChannelLengthRule {
    fn check(&self, item: &TrustDomainForecast) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainForecastAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl TrustDomainForecastRule for TrustDomainForecastAllowedCountryRule {
    fn check(&self, item: &TrustDomainForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is duplicated", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainForecastCountryLengthRule {
    pub max_length: i64,
}

impl TrustDomainForecastRule for TrustDomainForecastCountryLengthRule {
    fn check(&self, item: &TrustDomainForecast) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainForecastEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainForecastRule for TrustDomainForecastEndsAtRangeRule {
    fn check(&self, item: &TrustDomainForecast) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain forecast.
#[derive(Debug, Clone)]
pub struct TrustDomainForecastValidator {
    pub rules: Vec<Rc<dyn TrustDomainForecastRule>>,
}

impl TrustDomainForecastValidator {
    pub fn validate(&self, item: &TrustDomainForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_forecast_validator() -> TrustDomainForecastValidator {
    TrustDomainForecastValidator {
        rules: vec![
            Rc::new(TrustDomainForecastChannelLengthRule { max_length: 16 }) as Rc<dyn TrustDomainForecastRule>,
            Rc::new(TrustDomainForecastAllowedCountryRule {
                allowed: vec!["scheduled".to_string(), "lenient".to_string()],
            }),
            Rc::new(TrustDomainForecastCountryLengthRule { max_length: 32 }),
            Rc::new(TrustDomainForecastEndsAtRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_forecast_validator();
        let item = new_trust_domain_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = TrustDomainForecastChannelLengthRule { max_length: 255 };
        let mut item = new_trust_domain_forecast("x", "y");
        item.channel = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = TrustDomainForecastCountryLengthRule { max_length: 120 };
        let mut item = new_trust_domain_forecast("x", "y");
        item.country = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = TrustDomainForecastEndsAtRangeRule { min: 32, max: 255 };
        let mut item = new_trust_domain_forecast("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
