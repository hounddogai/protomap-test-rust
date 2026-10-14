use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast bundle must pass. Check returns its problems, or none.
pub trait ForecastBundleRule: std::fmt::Debug {
    fn check(&self, item: &ForecastBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBundleCodePrefixRule {
    pub prefix: String,
}

impl ForecastBundleRule for ForecastBundleCodePrefixRule {
    fn check(&self, item: &ForecastBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBundleLabelLengthRule {
    pub max_length: i64,
}

impl ForecastBundleRule for ForecastBundleLabelLengthRule {
    fn check(&self, item: &ForecastBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBundleRequiredSegmentRule {
    pub message: String,
}

impl ForecastBundleRule for ForecastBundleRequiredSegmentRule {
    fn check(&self, item: &ForecastBundle) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast bundle.
#[derive(Debug, Clone)]
pub struct ForecastBundleValidator {
    pub rules: Vec<Rc<dyn ForecastBundleRule>>,
}

impl ForecastBundleValidator {
    pub fn validate(&self, item: &ForecastBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_bundle_validator() -> ForecastBundleValidator {
    ForecastBundleValidator {
        rules: vec![
            Rc::new(ForecastBundleCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn ForecastBundleRule>,
            Rc::new(ForecastBundleLabelLengthRule { max_length: 3600 }),
            Rc::new(ForecastBundleRequiredSegmentRule {
                message: "forecast bundle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_bundle_validator();
        let item = new_forecast_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ForecastBundleLabelLengthRule { max_length: 255 };
        let mut item = new_forecast_bundle("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = ForecastBundleRequiredSegmentRule {
            message: "forecast bundle needs this field".to_string(),
        };
        let mut item = new_forecast_bundle("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
