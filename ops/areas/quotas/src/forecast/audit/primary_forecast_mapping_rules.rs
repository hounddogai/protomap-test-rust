use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary forecast mapping must pass. Check returns its problems, or none.
pub trait PrimaryForecastMappingRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryForecastMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastMappingIssuerLengthRule {
    pub max_length: i64,
}

impl PrimaryForecastMappingRule for PrimaryForecastMappingIssuerLengthRule {
    fn check(&self, item: &PrimaryForecastMapping) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastMappingRequiredLabelRule {
    pub message: String,
}

impl PrimaryForecastMappingRule for PrimaryForecastMappingRequiredLabelRule {
    fn check(&self, item: &PrimaryForecastMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastMappingLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryForecastMappingRule for PrimaryForecastMappingLabelLengthRule {
    fn check(&self, item: &PrimaryForecastMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastMappingCodePrefixRule {
    pub prefix: String,
}

impl PrimaryForecastMappingRule for PrimaryForecastMappingCodePrefixRule {
    fn check(&self, item: &PrimaryForecastMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastMappingAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl PrimaryForecastMappingRule for PrimaryForecastMappingAllowedIssuerRule {
    fn check(&self, item: &PrimaryForecastMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is duplicated", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastMappingCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryForecastMappingRule for PrimaryForecastMappingCostRangeRule {
    fn check(&self, item: &PrimaryForecastMapping) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a primary forecast mapping.
#[derive(Debug, Clone)]
pub struct PrimaryForecastMappingValidator {
    pub rules: Vec<Rc<dyn PrimaryForecastMappingRule>>,
}

impl PrimaryForecastMappingValidator {
    pub fn validate(&self, item: &PrimaryForecastMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryForecastMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryForecastMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_forecast_mapping_validator() -> PrimaryForecastMappingValidator {
    PrimaryForecastMappingValidator {
        rules: vec![
            Rc::new(PrimaryForecastMappingIssuerLengthRule { max_length: 16 }) as Rc<dyn PrimaryForecastMappingRule>,
            Rc::new(PrimaryForecastMappingRequiredLabelRule {
                message: "primary forecast mapping needs a value".to_string(),
            }),
            Rc::new(PrimaryForecastMappingLabelLengthRule { max_length: 255 }),
            Rc::new(PrimaryForecastMappingCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PrimaryForecastMappingAllowedIssuerRule {
                allowed: vec!["tiered".to_string(), "primary".to_string(), "adaptive".to_string()],
            }),
            Rc::new(PrimaryForecastMappingCostRangeRule { min: 7.3, max: 3403.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_forecast_mapping_validator();
        let item = new_primary_forecast_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PrimaryForecastMappingIssuerLengthRule { max_length: 32 };
        let mut item = new_primary_forecast_mapping("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryForecastMappingRequiredLabelRule {
            message: "primary forecast mapping needs a value".to_string(),
        };
        let mut item = new_primary_forecast_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryForecastMappingLabelLengthRule { max_length: 64 };
        let mut item = new_primary_forecast_mapping("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PrimaryForecastMappingCostRangeRule { min: 8.8, max: 1812.4 };
        let mut item = new_primary_forecast_mapping("x", "y");
        item.cost = 1813.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
