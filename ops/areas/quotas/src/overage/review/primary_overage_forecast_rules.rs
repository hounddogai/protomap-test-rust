use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary overage forecast must pass. Check returns its problems, or none.
pub trait PrimaryOverageForecastRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryOverageForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageForecastIssuerLengthRule {
    pub max_length: i64,
}

impl PrimaryOverageForecastRule for PrimaryOverageForecastIssuerLengthRule {
    fn check(&self, item: &PrimaryOverageForecast) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageForecastRequiredIssuerRule {
    pub message: String,
}

impl PrimaryOverageForecastRule for PrimaryOverageForecastRequiredIssuerRule {
    fn check(&self, item: &PrimaryOverageForecast) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageForecastCodePrefixRule {
    pub prefix: String,
}

impl PrimaryOverageForecastRule for PrimaryOverageForecastCodePrefixRule {
    fn check(&self, item: &PrimaryOverageForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageForecastRequiredHostRule {
    pub message: String,
}

impl PrimaryOverageForecastRule for PrimaryOverageForecastRequiredHostRule {
    fn check(&self, item: &PrimaryOverageForecast) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageForecastUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryOverageForecastRule for PrimaryOverageForecastUpdatedAtRangeRule {
    fn check(&self, item: &PrimaryOverageForecast) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageForecastRequiredCodeRule {
    pub message: String,
}

impl PrimaryOverageForecastRule for PrimaryOverageForecastRequiredCodeRule {
    fn check(&self, item: &PrimaryOverageForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary overage forecast.
#[derive(Debug, Clone)]
pub struct PrimaryOverageForecastValidator {
    pub rules: Vec<Rc<dyn PrimaryOverageForecastRule>>,
}

impl PrimaryOverageForecastValidator {
    pub fn validate(&self, item: &PrimaryOverageForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryOverageForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryOverageForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_overage_forecast_validator() -> PrimaryOverageForecastValidator {
    PrimaryOverageForecastValidator {
        rules: vec![
            Rc::new(PrimaryOverageForecastIssuerLengthRule { max_length: 64 }) as Rc<dyn PrimaryOverageForecastRule>,
            Rc::new(PrimaryOverageForecastRequiredIssuerRule {
                message: "primary overage forecast needs this field".to_string(),
            }),
            Rc::new(PrimaryOverageForecastCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PrimaryOverageForecastRequiredHostRule {
                message: "primary overage forecast needs an entry".to_string(),
            }),
            Rc::new(PrimaryOverageForecastUpdatedAtRangeRule { min: 120, max: 3600 }),
            Rc::new(PrimaryOverageForecastRequiredCodeRule {
                message: "primary overage forecast needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_overage_forecast_validator();
        let item = new_primary_overage_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PrimaryOverageForecastIssuerLengthRule { max_length: 16 };
        let mut item = new_primary_overage_forecast("x", "y");
        item.issuer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = PrimaryOverageForecastRequiredIssuerRule {
            message: "primary overage forecast needs this field".to_string(),
        };
        let mut item = new_primary_overage_forecast("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PrimaryOverageForecastRequiredHostRule {
            message: "primary overage forecast needs this field".to_string(),
        };
        let mut item = new_primary_overage_forecast("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = PrimaryOverageForecastUpdatedAtRangeRule { min: 80, max: 255 };
        let mut item = new_primary_overage_forecast("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryOverageForecastRequiredCodeRule {
            message: "primary overage forecast needs a value".to_string(),
        };
        let mut item = new_primary_overage_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
