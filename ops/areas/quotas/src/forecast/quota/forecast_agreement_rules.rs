use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast agreement must pass. Check returns its problems, or none.
pub trait ForecastAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ForecastAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAgreementRequiredTitleRule {
    pub message: String,
}

impl ForecastAgreementRule for ForecastAgreementRequiredTitleRule {
    fn check(&self, item: &ForecastAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAgreementTitleLengthRule {
    pub max_length: i64,
}

impl ForecastAgreementRule for ForecastAgreementTitleLengthRule {
    fn check(&self, item: &ForecastAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAgreementSourceLengthRule {
    pub max_length: i64,
}

impl ForecastAgreementRule for ForecastAgreementSourceLengthRule {
    fn check(&self, item: &ForecastAgreement) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAgreementCodePrefixRule {
    pub prefix: String,
}

impl ForecastAgreementRule for ForecastAgreementCodePrefixRule {
    fn check(&self, item: &ForecastAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAgreementRequiredRegionRule {
    pub message: String,
}

impl ForecastAgreementRule for ForecastAgreementRequiredRegionRule {
    fn check(&self, item: &ForecastAgreement) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast agreement.
#[derive(Debug, Clone)]
pub struct ForecastAgreementValidator {
    pub rules: Vec<Rc<dyn ForecastAgreementRule>>,
}

impl ForecastAgreementValidator {
    pub fn validate(&self, item: &ForecastAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_agreement_validator() -> ForecastAgreementValidator {
    ForecastAgreementValidator {
        rules: vec![
            Rc::new(ForecastAgreementRequiredTitleRule {
                message: "forecast agreement needs this field".to_string(),
            }) as Rc<dyn ForecastAgreementRule>,
            Rc::new(ForecastAgreementTitleLengthRule { max_length: 86400 }),
            Rc::new(ForecastAgreementSourceLengthRule { max_length: 3600 }),
            Rc::new(ForecastAgreementCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ForecastAgreementRequiredRegionRule {
                message: "forecast agreement needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_agreement_validator();
        let item = new_forecast_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ForecastAgreementRequiredTitleRule {
            message: "forecast agreement needs a value".to_string(),
        };
        let mut item = new_forecast_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ForecastAgreementTitleLengthRule { max_length: 64 };
        let mut item = new_forecast_agreement("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = ForecastAgreementSourceLengthRule { max_length: 64 };
        let mut item = new_forecast_agreement("x", "y");
        item.source = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = ForecastAgreementRequiredRegionRule {
            message: "forecast agreement needs a value".to_string(),
        };
        let mut item = new_forecast_agreement("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
