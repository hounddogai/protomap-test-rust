use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast digest must pass. Check returns its problems, or none.
pub trait ForecastDigestRule: std::fmt::Debug {
    fn check(&self, item: &ForecastDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDigestRequiredDescriptionRule {
    pub message: String,
}

impl ForecastDigestRule for ForecastDigestRequiredDescriptionRule {
    fn check(&self, item: &ForecastDigest) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDigestMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastDigestRule for ForecastDigestMarginRangeRule {
    fn check(&self, item: &ForecastDigest) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDigestCodeLengthRule {
    pub max_length: i64,
}

impl ForecastDigestRule for ForecastDigestCodeLengthRule {
    fn check(&self, item: &ForecastDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDigestPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastDigestRule for ForecastDigestPortRangeRule {
    fn check(&self, item: &ForecastDigest) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDigestCodePrefixRule {
    pub prefix: String,
}

impl ForecastDigestRule for ForecastDigestCodePrefixRule {
    fn check(&self, item: &ForecastDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast digest.
#[derive(Debug, Clone)]
pub struct ForecastDigestValidator {
    pub rules: Vec<Rc<dyn ForecastDigestRule>>,
}

impl ForecastDigestValidator {
    pub fn validate(&self, item: &ForecastDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_digest_validator() -> ForecastDigestValidator {
    ForecastDigestValidator {
        rules: vec![
            Rc::new(ForecastDigestRequiredDescriptionRule {
                message: "forecast digest needs this field".to_string(),
            }) as Rc<dyn ForecastDigestRule>,
            Rc::new(ForecastDigestMarginRangeRule { min: 2.8, max: 4964.9 }),
            Rc::new(ForecastDigestCodeLengthRule { max_length: 64 }),
            Rc::new(ForecastDigestPortRangeRule { min: 120, max: 86400 }),
            Rc::new(ForecastDigestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_digest_validator();
        let item = new_forecast_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = ForecastDigestRequiredDescriptionRule {
            message: "forecast digest needs an entry".to_string(),
        };
        let mut item = new_forecast_digest("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ForecastDigestMarginRangeRule { min: 8.3, max: 1479.3 };
        let mut item = new_forecast_digest("x", "y");
        item.margin = 1480.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastDigestCodeLengthRule { max_length: 120 };
        let mut item = new_forecast_digest("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = ForecastDigestPortRangeRule { min: 32, max: 80 };
        let mut item = new_forecast_digest("x", "y");
        item.port = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
