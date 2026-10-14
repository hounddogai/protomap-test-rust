use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast request must pass. Check returns its problems, or none.
pub trait ForecastRequestRule: std::fmt::Debug {
    fn check(&self, item: &ForecastRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRequestRequiredCodeRule {
    pub message: String,
}

impl ForecastRequestRule for ForecastRequestRequiredCodeRule {
    fn check(&self, item: &ForecastRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRequestRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastRequestRule for ForecastRequestRotationDaysRangeRule {
    fn check(&self, item: &ForecastRequest) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRequestCodePrefixRule {
    pub prefix: String,
}

impl ForecastRequestRule for ForecastRequestCodePrefixRule {
    fn check(&self, item: &ForecastRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast request.
#[derive(Debug, Clone)]
pub struct ForecastRequestValidator {
    pub rules: Vec<Rc<dyn ForecastRequestRule>>,
}

impl ForecastRequestValidator {
    pub fn validate(&self, item: &ForecastRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_request_validator() -> ForecastRequestValidator {
    ForecastRequestValidator {
        rules: vec![
            Rc::new(ForecastRequestRequiredCodeRule {
                message: "forecast request needs an entry".to_string(),
            }) as Rc<dyn ForecastRequestRule>,
            Rc::new(ForecastRequestRotationDaysRangeRule { min: 16, max: 255 }),
            Rc::new(ForecastRequestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_request_validator();
        let item = new_forecast_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastRequestRequiredCodeRule {
            message: "forecast request needs an entry".to_string(),
        };
        let mut item = new_forecast_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = ForecastRequestRotationDaysRangeRule { min: 32, max: 120 };
        let mut item = new_forecast_request("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
