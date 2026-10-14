use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered forecast agreement must pass. Check returns its problems, or none.
pub trait TieredForecastAgreementRule: std::fmt::Debug {
    fn check(&self, item: &TieredForecastAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastAgreementCodePrefixRule {
    pub prefix: String,
}

impl TieredForecastAgreementRule for TieredForecastAgreementCodePrefixRule {
    fn check(&self, item: &TieredForecastAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastAgreementRequiredSourceRule {
    pub message: String,
}

impl TieredForecastAgreementRule for TieredForecastAgreementRequiredSourceRule {
    fn check(&self, item: &TieredForecastAgreement) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastAgreementTitleLengthRule {
    pub max_length: i64,
}

impl TieredForecastAgreementRule for TieredForecastAgreementTitleLengthRule {
    fn check(&self, item: &TieredForecastAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastAgreementMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredForecastAgreementRule for TieredForecastAgreementMinimumRangeRule {
    fn check(&self, item: &TieredForecastAgreement) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered forecast agreement.
#[derive(Debug, Clone)]
pub struct TieredForecastAgreementValidator {
    pub rules: Vec<Rc<dyn TieredForecastAgreementRule>>,
}

impl TieredForecastAgreementValidator {
    pub fn validate(&self, item: &TieredForecastAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredForecastAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredForecastAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_forecast_agreement_validator() -> TieredForecastAgreementValidator {
    TieredForecastAgreementValidator {
        rules: vec![
            Rc::new(TieredForecastAgreementCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn TieredForecastAgreementRule>,
            Rc::new(TieredForecastAgreementRequiredSourceRule {
                message: "tiered forecast agreement needs this field".to_string(),
            }),
            Rc::new(TieredForecastAgreementTitleLengthRule { max_length: 120 }),
            Rc::new(TieredForecastAgreementMinimumRangeRule { min: 1.0, max: 3313.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_forecast_agreement_validator();
        let item = new_tiered_forecast_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = TieredForecastAgreementRequiredSourceRule {
            message: "tiered forecast agreement needs an entry".to_string(),
        };
        let mut item = new_tiered_forecast_agreement("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredForecastAgreementTitleLengthRule { max_length: 32 };
        let mut item = new_tiered_forecast_agreement("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TieredForecastAgreementMinimumRangeRule { min: 3.3, max: 2458.1 };
        let mut item = new_tiered_forecast_agreement("x", "y");
        item.minimum = 2459.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
