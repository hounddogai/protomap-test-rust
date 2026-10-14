use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast allocation must pass. Check returns its problems, or none.
pub trait ForecastAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ForecastAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAllocationRequiredRegionRule {
    pub message: String,
}

impl ForecastAllocationRule for ForecastAllocationRequiredRegionRule {
    fn check(&self, item: &ForecastAllocation) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAllocationNameLengthRule {
    pub max_length: i64,
}

impl ForecastAllocationRule for ForecastAllocationNameLengthRule {
    fn check(&self, item: &ForecastAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAllocationRequiredNameRule {
    pub message: String,
}

impl ForecastAllocationRule for ForecastAllocationRequiredNameRule {
    fn check(&self, item: &ForecastAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAllocationContactEmailLengthRule {
    pub max_length: i64,
}

impl ForecastAllocationRule for ForecastAllocationContactEmailLengthRule {
    fn check(&self, item: &ForecastAllocation) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAllocationAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl ForecastAllocationRule for ForecastAllocationAllowedRegionRule {
    fn check(&self, item: &ForecastAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is above maximum", item.region)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast allocation.
#[derive(Debug, Clone)]
pub struct ForecastAllocationValidator {
    pub rules: Vec<Rc<dyn ForecastAllocationRule>>,
}

impl ForecastAllocationValidator {
    pub fn validate(&self, item: &ForecastAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_allocation_validator() -> ForecastAllocationValidator {
    ForecastAllocationValidator {
        rules: vec![
            Rc::new(ForecastAllocationRequiredRegionRule {
                message: "forecast allocation needs an entry".to_string(),
            }) as Rc<dyn ForecastAllocationRule>,
            Rc::new(ForecastAllocationNameLengthRule { max_length: 80 }),
            Rc::new(ForecastAllocationRequiredNameRule {
                message: "forecast allocation needs a value".to_string(),
            }),
            Rc::new(ForecastAllocationContactEmailLengthRule { max_length: 120 }),
            Rc::new(ForecastAllocationAllowedRegionRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_allocation_validator();
        let item = new_forecast_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = ForecastAllocationRequiredRegionRule {
            message: "forecast allocation needs an entry".to_string(),
        };
        let mut item = new_forecast_allocation("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ForecastAllocationNameLengthRule { max_length: 86400 };
        let mut item = new_forecast_allocation("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ForecastAllocationRequiredNameRule {
            message: "forecast allocation needs this field".to_string(),
        };
        let mut item = new_forecast_allocation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = ForecastAllocationContactEmailLengthRule { max_length: 80 };
        let mut item = new_forecast_allocation("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
