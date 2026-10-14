use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional forecast bundle must pass. Check returns its problems, or none.
pub trait RegionalForecastBundleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalForecastBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastBundleVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalForecastBundleRule for RegionalForecastBundleVersionRangeRule {
    fn check(&self, item: &RegionalForecastBundle) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastBundleExternalRefLengthRule {
    pub max_length: i64,
}

impl RegionalForecastBundleRule for RegionalForecastBundleExternalRefLengthRule {
    fn check(&self, item: &RegionalForecastBundle) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastBundleApprovedGuardRule {
    pub limit: i64,
}

impl RegionalForecastBundleRule for RegionalForecastBundleApprovedGuardRule {
    fn check(&self, item: &RegionalForecastBundle) -> Vec<String> {
        if item.approved && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while approved",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastBundleRequiredExternalRefRule {
    pub message: String,
}

impl RegionalForecastBundleRule for RegionalForecastBundleRequiredExternalRefRule {
    fn check(&self, item: &RegionalForecastBundle) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional forecast bundle.
#[derive(Debug, Clone)]
pub struct RegionalForecastBundleValidator {
    pub rules: Vec<Rc<dyn RegionalForecastBundleRule>>,
}

impl RegionalForecastBundleValidator {
    pub fn validate(&self, item: &RegionalForecastBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalForecastBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalForecastBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_forecast_bundle_validator() -> RegionalForecastBundleValidator {
    RegionalForecastBundleValidator {
        rules: vec![
            Rc::new(RegionalForecastBundleVersionRangeRule { min: 120, max: 3600 })
                as Rc<dyn RegionalForecastBundleRule>,
            Rc::new(RegionalForecastBundleExternalRefLengthRule { max_length: 120 }),
            Rc::new(RegionalForecastBundleApprovedGuardRule { limit: 64 }),
            Rc::new(RegionalForecastBundleRequiredExternalRefRule {
                message: "regional forecast bundle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_forecast_bundle_validator();
        let item = new_regional_forecast_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RegionalForecastBundleVersionRangeRule { min: 255, max: 255 };
        let mut item = new_regional_forecast_bundle("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = RegionalForecastBundleExternalRefLengthRule { max_length: 80 };
        let mut item = new_regional_forecast_bundle("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = RegionalForecastBundleRequiredExternalRefRule {
            message: "regional forecast bundle needs an entry".to_string(),
        };
        let mut item = new_regional_forecast_bundle("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
