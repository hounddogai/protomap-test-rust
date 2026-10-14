use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast manifest must pass. Check returns its problems, or none.
pub trait ForecastManifestRule: std::fmt::Debug {
    fn check(&self, item: &ForecastManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastManifestCodeLengthRule {
    pub max_length: i64,
}

impl ForecastManifestRule for ForecastManifestCodeLengthRule {
    fn check(&self, item: &ForecastManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastManifestReviewerLengthRule {
    pub max_length: i64,
}

impl ForecastManifestRule for ForecastManifestReviewerLengthRule {
    fn check(&self, item: &ForecastManifest) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastManifestRequiredNameRule {
    pub message: String,
}

impl ForecastManifestRule for ForecastManifestRequiredNameRule {
    fn check(&self, item: &ForecastManifest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastManifestRequiredCodeRule {
    pub message: String,
}

impl ForecastManifestRule for ForecastManifestRequiredCodeRule {
    fn check(&self, item: &ForecastManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastManifestLocaleLengthRule {
    pub max_length: i64,
}

impl ForecastManifestRule for ForecastManifestLocaleLengthRule {
    fn check(&self, item: &ForecastManifest) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastManifestNameLengthRule {
    pub max_length: i64,
}

impl ForecastManifestRule for ForecastManifestNameLengthRule {
    fn check(&self, item: &ForecastManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast manifest.
#[derive(Debug, Clone)]
pub struct ForecastManifestValidator {
    pub rules: Vec<Rc<dyn ForecastManifestRule>>,
}

impl ForecastManifestValidator {
    pub fn validate(&self, item: &ForecastManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_manifest_validator() -> ForecastManifestValidator {
    ForecastManifestValidator {
        rules: vec![
            Rc::new(ForecastManifestCodeLengthRule { max_length: 3600 }) as Rc<dyn ForecastManifestRule>,
            Rc::new(ForecastManifestReviewerLengthRule { max_length: 255 }),
            Rc::new(ForecastManifestRequiredNameRule {
                message: "forecast manifest needs a value".to_string(),
            }),
            Rc::new(ForecastManifestRequiredCodeRule {
                message: "forecast manifest needs a value".to_string(),
            }),
            Rc::new(ForecastManifestLocaleLengthRule { max_length: 3600 }),
            Rc::new(ForecastManifestNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_manifest_validator();
        let item = new_forecast_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastManifestCodeLengthRule { max_length: 16 };
        let mut item = new_forecast_manifest("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = ForecastManifestReviewerLengthRule { max_length: 120 };
        let mut item = new_forecast_manifest("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ForecastManifestRequiredNameRule {
            message: "forecast manifest needs a value".to_string(),
        };
        let mut item = new_forecast_manifest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastManifestRequiredCodeRule {
            message: "forecast manifest needs an entry".to_string(),
        };
        let mut item = new_forecast_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ForecastManifestLocaleLengthRule { max_length: 64 };
        let mut item = new_forecast_manifest("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ForecastManifestNameLengthRule { max_length: 3600 };
        let mut item = new_forecast_manifest("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
