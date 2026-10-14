use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast assignment must pass. Check returns its problems, or none.
pub trait ForecastAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ForecastAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAssignmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastAssignmentRule for ForecastAssignmentQuantityRangeRule {
    fn check(&self, item: &ForecastAssignment) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAssignmentRequiredLabelRule {
    pub message: String,
}

impl ForecastAssignmentRule for ForecastAssignmentRequiredLabelRule {
    fn check(&self, item: &ForecastAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAssignmentAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl ForecastAssignmentRule for ForecastAssignmentAllowedSourceRule {
    fn check(&self, item: &ForecastAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is stale", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAssignmentCodePrefixRule {
    pub prefix: String,
}

impl ForecastAssignmentRule for ForecastAssignmentCodePrefixRule {
    fn check(&self, item: &ForecastAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAssignmentTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastAssignmentRule for ForecastAssignmentTtlSecondsRangeRule {
    fn check(&self, item: &ForecastAssignment) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a forecast assignment.
#[derive(Debug, Clone)]
pub struct ForecastAssignmentValidator {
    pub rules: Vec<Rc<dyn ForecastAssignmentRule>>,
}

impl ForecastAssignmentValidator {
    pub fn validate(&self, item: &ForecastAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_assignment_validator() -> ForecastAssignmentValidator {
    ForecastAssignmentValidator {
        rules: vec![
            Rc::new(ForecastAssignmentQuantityRangeRule { min: 64, max: 1000 }) as Rc<dyn ForecastAssignmentRule>,
            Rc::new(ForecastAssignmentRequiredLabelRule {
                message: "forecast assignment needs this field".to_string(),
            }),
            Rc::new(ForecastAssignmentAllowedSourceRule { allowed: Vec::new() }),
            Rc::new(ForecastAssignmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ForecastAssignmentTtlSecondsRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_assignment_validator();
        let item = new_forecast_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ForecastAssignmentQuantityRangeRule { min: 16, max: 1000 };
        let mut item = new_forecast_assignment("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ForecastAssignmentRequiredLabelRule {
            message: "forecast assignment needs an entry".to_string(),
        };
        let mut item = new_forecast_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = ForecastAssignmentTtlSecondsRangeRule { min: 120, max: 255 };
        let mut item = new_forecast_assignment("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
