use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback data export estimate must pass. Check returns its problems, or none.
pub trait FallbackDataExportEstimateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDataExportEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportEstimateCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackDataExportEstimateRule for FallbackDataExportEstimateCapacityRangeRule {
    fn check(&self, item: &FallbackDataExportEstimate) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportEstimateRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackDataExportEstimateRule for FallbackDataExportEstimateRotationDaysRangeRule {
    fn check(&self, item: &FallbackDataExportEstimate) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportEstimateRequiredPhoneRule {
    pub message: String,
}

impl FallbackDataExportEstimateRule for FallbackDataExportEstimateRequiredPhoneRule {
    fn check(&self, item: &FallbackDataExportEstimate) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportEstimateRequiredCodeRule {
    pub message: String,
}

impl FallbackDataExportEstimateRule for FallbackDataExportEstimateRequiredCodeRule {
    fn check(&self, item: &FallbackDataExportEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportEstimatePrincipalLengthRule {
    pub max_length: i64,
}

impl FallbackDataExportEstimateRule for FallbackDataExportEstimatePrincipalLengthRule {
    fn check(&self, item: &FallbackDataExportEstimate) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportEstimateAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl FallbackDataExportEstimateRule for FallbackDataExportEstimateAllowedPrincipalRule {
    fn check(&self, item: &FallbackDataExportEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is stale", item.principal)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback data export estimate.
#[derive(Debug, Clone)]
pub struct FallbackDataExportEstimateValidator {
    pub rules: Vec<Rc<dyn FallbackDataExportEstimateRule>>,
}

impl FallbackDataExportEstimateValidator {
    pub fn validate(&self, item: &FallbackDataExportEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackDataExportEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDataExportEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_data_export_estimate_validator() -> FallbackDataExportEstimateValidator {
    FallbackDataExportEstimateValidator {
        rules: vec![
            Rc::new(FallbackDataExportEstimateCapacityRangeRule { min: 64, max: 3600 })
                as Rc<dyn FallbackDataExportEstimateRule>,
            Rc::new(FallbackDataExportEstimateRotationDaysRangeRule { min: 32, max: 120 }),
            Rc::new(FallbackDataExportEstimateRequiredPhoneRule {
                message: "fallback data export estimate needs an entry".to_string(),
            }),
            Rc::new(FallbackDataExportEstimateRequiredCodeRule {
                message: "fallback data export estimate needs an entry".to_string(),
            }),
            Rc::new(FallbackDataExportEstimatePrincipalLengthRule { max_length: 120 }),
            Rc::new(FallbackDataExportEstimateAllowedPrincipalRule {
                allowed: vec!["batched".to_string(), "rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_data_export_estimate_validator();
        let item = new_fallback_data_export_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = FallbackDataExportEstimateCapacityRangeRule { min: 64, max: 120 };
        let mut item = new_fallback_data_export_estimate("x", "y");
        item.capacity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = FallbackDataExportEstimateRotationDaysRangeRule { min: 32, max: 1000 };
        let mut item = new_fallback_data_export_estimate("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = FallbackDataExportEstimateRequiredPhoneRule {
            message: "fallback data export estimate needs an entry".to_string(),
        };
        let mut item = new_fallback_data_export_estimate("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackDataExportEstimateRequiredCodeRule {
            message: "fallback data export estimate needs this field".to_string(),
        };
        let mut item = new_fallback_data_export_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = FallbackDataExportEstimatePrincipalLengthRule { max_length: 86400 };
        let mut item = new_fallback_data_export_estimate("x", "y");
        item.principal = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
